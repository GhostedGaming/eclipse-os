use crate::pc_speaker;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use crate::text_editor::express_editor::EDITOR_DATA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,     // Full square/block cursor (inverts colors)
    Underline, // Underline cursor (changes fg color)
    Invert,    // Invert colors (like block, but can be styled differently)
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0, // Start at top row like a normal terminal
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        cursor_visible: true,
        cursor_color: ColorCode::new(Color::Black, Color::LightGray),
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
pub const BUFFER_HEIGHT: usize = 25;

/// The width of the text buffer (normally 80 columns).
pub const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    pub cursor_visible: bool,
    cursor_color: ColorCode,
    cursor_style: CursorStyle,
    saved_cursor_char: Option<ScreenChar>,
}

impl Writer {
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    /// Safely clamp coordinates to buffer bounds
    fn clamp_position(&self, row: usize, col: usize) -> (usize, usize) {
        let safe_row = row.min(BUFFER_HEIGHT - 1);
        let safe_col = col.min(BUFFER_WIDTH - 1);
        (safe_row, safe_col)
    }

    /// Get current cursor position
    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.row_position, self.column_position)
    }

    /// Set cursor position with bounds checking
    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        self.erase_cursor();
        let (safe_row, safe_col) = self.clamp_position(row, col);
        self.row_position = safe_row;
        self.column_position = safe_col;
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    /// Move cursor with bounds checking
    pub fn move_cursor_relative(&mut self, row_delta: i32, col_delta: i32) {
        self.erase_cursor();
        
        // Calculate new position
        let new_row = (self.row_position as i32 + row_delta).max(0) as usize;
        let new_col = (self.column_position as i32 + col_delta).max(0) as usize;
        
        // Clamp to bounds
        let (safe_row, safe_col) = self.clamp_position(new_row, new_col);
        self.row_position = safe_row;
        self.column_position = safe_col;
        
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.erase_cursor();

        match byte {
            b'\n' => self.new_line(),
            b'\r' => {
                // Carriage return - move to beginning of current line
                self.column_position = 0;
            }
            b'\t' => {
                // Tab - move to next tab stop (every 4 characters)
                let tab_size = 4;
                let new_col = ((self.column_position / tab_size) + 1) * tab_size;
                if new_col >= BUFFER_WIDTH {
                    self.new_line();
                } else {
                    self.column_position = new_col;
                }
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let color_code = self.color_code;
                self.buffer.chars[self.row_position][self.column_position].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }

        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\r' | b'\t' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // Replacement character
            }
        }
    }

    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position >= BUFFER_HEIGHT - 1 {
            // Scroll up when we reach the bottom
            self.scroll_up();
        } else {
            self.row_position += 1;
        }
    }

    /// Scroll the entire buffer up by one line
    fn scroll_up(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        // Stay at the last row after scrolling
        self.row_position = BUFFER_HEIGHT - 1;
    }

    fn clear_row(&mut self, row: usize) {
        if row >= BUFFER_HEIGHT {
            return;
        }
        
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
            self.erase_cursor();
            self.column_position -= 1;

            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            self.buffer.chars[self.row_position][self.column_position].write(blank);

            if self.cursor_visible {
                self.draw_cursor();
            }
        } else {
            self.erase_cursor();
            self.row_position -= 1;

            // if self.row_position < 1 {
            //     self.scroll_up();
            // }

            if self.cursor_visible {
                self.draw_cursor();
            }
        }
    }

    /// Get column position (for backward compatibility)
    pub fn column_position(&self) -> usize {
        self.column_position
    }

    /// Get row position
    pub fn row_position(&self) -> usize {
        self.row_position
    }

    pub fn draw_cursor(&mut self) {
        let (row, col) = self.clamp_position(self.row_position, self.column_position);
        
        // Save the current character under the cursor if not already saved
        if self.saved_cursor_char.is_none() {
            self.saved_cursor_char = Some(self.buffer.chars[row][col].read());
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
        let (row, col) = self.clamp_position(self.row_position, self.column_position);
        if let Some(orig) = self.saved_cursor_char.take() {
            self.buffer.chars[row][col].write(orig);
        }
    }

    pub fn toggle_cursor(&mut self) {
        if self.cursor_visible {
            self.erase_cursor();
        } else {
            self.draw_cursor();
        }
        self.cursor_visible = !self.cursor_visible;
    }

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

    pub fn set_cursor_style(&mut self, style: CursorStyle) {
        self.erase_cursor();
        self.cursor_style = style;
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    /// Write character at specific position without moving cursor
    pub fn write_char_at(&mut self, row: usize, col: usize, ch: u8, color: ColorCode) {
        let (safe_row, safe_col) = self.clamp_position(row, col);
        self.buffer.chars[safe_row][safe_col].write(ScreenChar {
            ascii_character: ch,
            color_code: color,
        });
    }

    /// Read character at specific position
    pub fn read_char_at(&self, row: usize, col: usize) -> Option<ScreenChar> {
        if row < BUFFER_HEIGHT && col < BUFFER_WIDTH {
            Some(self.buffer.chars[row][col].read())
        } else {
            None
        }
    }

    /// Move cursor to specific column on current row
    pub fn move_cursor(&mut self, new_col: usize) {
        self.erase_cursor();
        self.column_position = new_col.min(BUFFER_WIDTH - 1);
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

// Global functions

pub fn set_color(foreground: Color, background: Color) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().set_color(foreground, background);
    });
}

pub fn get_cursor_position() -> (usize, usize) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let writer = WRITER.lock();
        writer.get_cursor_position()
    })
}

pub fn set_cursor_position(row: usize, col: usize) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.set_cursor_position(row, col);
    });
}

pub fn move_cursor_left(positions: usize) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        let current_pos = writer.column_position;
        if current_pos >= positions {
            writer.move_cursor(current_pos - positions);
        } else {
            writer.move_cursor(0);
        }
    });
}

pub fn move_cursor_right(positions: usize) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        let current_pos = writer.column_position;
        let new_pos = (current_pos + positions).min(BUFFER_WIDTH - 1);
        writer.move_cursor(new_pos);
    });
}

pub fn move_cursor_to_start_of_line() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.erase_cursor();
        writer.column_position = 0;
        if writer.cursor_visible {
            writer.draw_cursor();
        }
    });
}

pub fn move_cursor_to_end_of_line() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.erase_cursor();
        writer.column_position = BUFFER_WIDTH - 1;
        if writer.cursor_visible {
            writer.draw_cursor();
        }
    });
}

pub fn move_cursor_up(lines: usize) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.move_cursor_relative(-(lines as i32), 0);
    });
}

pub fn move_cursor_down(lines: usize) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.move_cursor_relative(lines as i32, 0);
    });
}

pub fn backspace() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().handle_backspace();
    });
}

pub fn set_cursor_style(style: CursorStyle) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().set_cursor_style(style);
    });
}

pub fn set_cursor_visibility(visible: bool) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().set_cursor_visibility(visible);
    });
}

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
        writer.set_cursor_position(0, 0); // Start at top-left like a normal terminal
        if writer.cursor_visible {
            writer.draw_cursor();
        }
    });
}

pub fn clear_line() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        let current_row = writer.row_position;
        writer.clear_row(current_row);
        writer.column_position = 0;
        if writer.cursor_visible {
            writer.draw_cursor();
        }
    });
}

pub fn write_char_at(row: usize, col: usize, ch: u8, foreground: Color, background: Color) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let color = ColorCode::new(foreground, background);
        WRITER.lock().write_char_at(row, col, ch, color);
    });
}

pub fn read_char_at(row: usize, col: usize) -> Option<(u8, Color, Color)> {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        if let Some(screen_char) = WRITER.lock().read_char_at(row, col) {
            Some((
                screen_char.ascii_character,
                screen_char.color_code.fg(),
                screen_char.color_code.bg(),
            ))
        } else {
            None
        }
    })
}

/// Print the given formatted string to the VGA text buffer
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut s = heapless::String::<256>::new();
        let _ = write!(&mut s, $($arg)*);
        $crate::serial::info(&s);
        $crate::vga_buffer::_print(format_args!($($arg)*));
    }};
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
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
            if let Some(screen_char) = writer.read_char_at(writer.row_position() - 1, i) {
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        }
    });
}

#[test_case]
fn test_cursor_movement() {
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        
        // Test bounds checking
        writer.set_cursor_position(0, 0);
        assert_eq!(writer.get_cursor_position(), (0, 0));
        
        writer.set_cursor_position(BUFFER_HEIGHT + 10, BUFFER_WIDTH + 10);
        assert_eq!(writer.get_cursor_position(), (BUFFER_HEIGHT - 1, BUFFER_WIDTH - 1));
        
        // Test relative movement
        writer.set_cursor_position(10, 10);
        writer.move_cursor_relative(-5, -5);
        assert_eq!(writer.get_cursor_position(), (5, 5));
        
        // Test bounds on relative movement
        writer.move_cursor_relative(-10, -10);
        assert_eq!(writer.get_cursor_position(), (0, 0));
    });
}
