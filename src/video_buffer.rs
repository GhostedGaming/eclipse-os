use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

/// VGA color palette.
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

/// Foreground and background color code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
    pub fn fg(&self) -> Color {
        Color::from_u8(self.0 & 0x0F)
    }
    pub fn bg(&self) -> Color {
        Color::from_u8((self.0 >> 4) & 0x0F)
    }
}

/// A single character cell in the VGA buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

/// VGA buffer size.
pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

/// The VGA text buffer.
#[repr(transparent)]
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Cursor style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Underline,
    Invert,
}

/// The main writer for the VGA buffer.
pub struct Writer {
    pub column_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
    pub cursor_visible: bool,
    pub cursor_style: CursorStyle,
    pub saved_cursor_char: Option<ScreenChar>,
}

lazy_static! {
    pub static ref VIDEO_WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        cursor_visible: true,
        cursor_style: CursorStyle::Block,
        saved_cursor_char: None,
    });
}

impl Writer {
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color_code = ColorCode::new(fg, bg);
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.erase_cursor();
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

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

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    pub fn draw_cursor(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        if col >= BUFFER_WIDTH { return; }
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
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        if col < BUFFER_WIDTH {
            if let Some(orig) = self.saved_cursor_char.take() {
                self.buffer.chars[row][col].write(orig);
            }
        }
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
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Public API

pub fn set_color(fg: Color, bg: Color) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        VIDEO_WRITER.lock().set_color(fg, bg);
    });
}

pub fn clear_screen() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        VIDEO_WRITER.lock().clear_screen();
    });
}

pub fn set_cursor_visibility(visible: bool) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        VIDEO_WRITER.lock().set_cursor_visibility(visible);
    });
}

pub fn set_cursor_style(style: CursorStyle) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        VIDEO_WRITER.lock().set_cursor_style(style);
    });
}

pub fn backspace() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = VIDEO_WRITER.lock();
        if writer.column_position > 0 {
            writer.erase_cursor();
            writer.column_position -= 1;
            let row = BUFFER_HEIGHT - 1;
            let col = writer.column_position;
            let color_code = writer.color_code;
            writer.buffer.chars[row][col].write(ScreenChar {
                ascii_character: b' ',
                color_code,
            });
            if writer.cursor_visible {
                writer.draw_cursor();
            }
        }
    });
}

#[macro_export]
macro_rules! vprint {
    ($($arg:tt)*) => ($crate::video_buffer::_vprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vprintln {
    () => ($crate::vprint!("\n"));
    ($($arg:tt)*) => ($crate::vprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _vprint(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        VIDEO_WRITER.lock().write_fmt(args).unwrap();
    });
}