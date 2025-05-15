use crate::sounds;
use crate::text_editor::express_editor::EDITOR_DATA;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,     // Full square/block cursor (inverts colors)
    Underline, // Underline cursor (changes fg color)
    Invert,    // Invert colors (like block, but can be styled differently)
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        cursor_visible: true,
        cursor_style: CursorStyle::Block,
        saved_cursor_char: None,
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

impl Color {
    fn from_u8(val: u8) -> Color {
        match val {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Purple,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::Pink,
            14 => Color::Yellow,
            15 => Color::White,
            _ => Color::White,
        }
    }
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
    fn fg(&self) -> Color {
        Color::from_u8(self.0 & 0x0F)
    }
    fn bg(&self) -> Color {
        Color::from_u8((self.0 >> 4) & 0x0F)
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
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    cursor_visible: bool,
    cursor_style: CursorStyle,
    saved_cursor_char: Option<ScreenChar>, // Store the char under the cursor
}

impl Writer {
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn set_background_color(&mut self, background: Color) {
        self.color_code = ColorCode::new(Color::White, background);
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.erase_cursor();

        match byte {
            b'\n' => {
                self.new_line();
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;

                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
            }
        }

        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    /// Writes the given ASCII string to the buffer.
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            if self.column_position > 78 {
                self.new_line();
            }
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
            self.column_position = 0;
        } else {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.column_position = 0;
        }
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
        } else {
            let editor_data_active = EDITOR_DATA.lock().active;
            if !editor_data_active {
                sounds::play_beep_for(10, 500);
            }
        }
    }

    pub fn column_position(&self) -> usize {
        self.column_position
    }

    pub fn move_cursor_up(&mut self, lines: usize) {
        if self.column_position > 0 {
            self.column_position = self.column_position.saturating_sub(lines);
        }
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.row_position, self.column_position)
    }

    pub fn draw_cursor(&mut self) {
        let row = self.row_position;
        let col = self.column_position;
        if col >= BUFFER_WIDTH || row >= BUFFER_HEIGHT {
            return;
        }

        if self.saved_cursor_char.is_none() {
            let existing_char = self.buffer.chars[row][col].read();
            let actual_char = if existing_char.ascii_character == 0 {
                ScreenChar {
                    ascii_character: b' ',
                    color_code: self.color_code,
                }
            } else {
                existing_char
            };
            self.saved_cursor_char = Some(actual_char);
        }

        let current_char = self.saved_cursor_char.unwrap();
        let cursor_char = match self.cursor_style {
            CursorStyle::Block | CursorStyle::Invert => {
                let fg = current_char.color_code.bg();
                let bg = current_char.color_code.fg();
                ScreenChar {
                    ascii_character: current_char.ascii_character,
                    color_code: ColorCode::new(fg, bg),
                }
            }
            CursorStyle::Underline => {
                let mut underline_code = current_char.color_code;
                underline_code.0 = (underline_code.0 & 0xF0) | (Color::Yellow as u8);
                ScreenChar {
                    ascii_character: current_char.ascii_character,
                    color_code: underline_code,
                }
            }
        };

        self.buffer.chars[row][col].write(cursor_char);
    }

    pub fn erase_cursor(&mut self) {
        let row = self.row_position;
        let col = self.column_position;
        if col < BUFFER_WIDTH && row < BUFFER_HEIGHT {
            if let Some(orig) = self.saved_cursor_char.take() {
                self.buffer.chars[row][col].write(orig);
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.erase_cursor();
        if self.column_position > 0 {
            self.column_position -= 1;
        } else if self.row_position > 0 {
            self.row_position -= 1;
            self.column_position = BUFFER_WIDTH - 1;
        }
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    pub fn move_cursor_right(&mut self) {
        self.erase_cursor();
        if self.column_position < BUFFER_WIDTH - 1 {
            self.column_position += 1;
        } else if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
            self.column_position = 0;
        }
        if self.cursor_visible {
            self.draw_cursor();
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
        self.erase_cursor();
        self.column_position = new_col;
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

pub fn move_cursor_up(lines: usize) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().move_cursor_up(lines);
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
        if writer.cursor_visible {
            writer.draw_cursor();
        }
    });
}

pub fn move_cursor_left() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().move_cursor_left();
    });
}

pub fn move_cursor_right() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().move_cursor_right();
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
