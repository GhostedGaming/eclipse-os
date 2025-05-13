use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,      // Full square/block cursor
    Underline,  // Underline cursor
    Invert,     // Current implementation (inverted colors)
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        framebuffer: unsafe { &mut *(0x1234_5678 as *mut Framebuffer) }, // Replace with actual framebuffer address
        cursor_visible: true,
        cursor_color: ColorCode::new(Color::Black, Color::LightGray), // Inverted colors for cursor
        cursor_style: CursorStyle::Block, // Default to block cursor
    });
}

/// The standard color palette
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
struct ColorCode(u32);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    fn new(foreground: Color, background: Color) -> ColorCode {
        // Assuming a 32-bit color encoding (ARGB)
        let fg = (foreground as u32) << 16; // Foreground color in bits 16-23
        let bg = (background as u32) << 24; // Background color in bits 24-31
        ColorCode(fg | bg)
    }
}

/// A structure representing the framebuffer.
#[repr(transparent)]
struct Framebuffer {
    pixels: [[u32; FRAMEBUFFER_WIDTH]; FRAMEBUFFER_HEIGHT],
}

/// The dimensions of the framebuffer (e.g., 1024x768).
const FRAMEBUFFER_WIDTH: usize = 1024;
const FRAMEBUFFER_HEIGHT: usize = 768;

/// A writer type that allows rendering text to the framebuffer.
pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    framebuffer: &'static mut Framebuffer,
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
                if self.column_position >= FRAMEBUFFER_WIDTH / FONT_WIDTH {
                    self.new_line();
                }

                self.draw_char(self.column_position, self.row_position, byte, self.color_code);
                self.column_position += 1;
            }
        }

        // Draw cursor after writing
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte), // Printable ASCII or newline
                _ => self.write_byte(0xfe),                  // Non-printable character
            }
        }
    }

    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position < FRAMEBUFFER_HEIGHT / FONT_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scroll the framebuffer up
            for y in 1..FRAMEBUFFER_HEIGHT / FONT_HEIGHT {
                for x in 0..FRAMEBUFFER_WIDTH / FONT_WIDTH {
                    let char_pixel = self.framebuffer.pixels[y * FONT_HEIGHT][x * FONT_WIDTH];
                    self.framebuffer.pixels[(y - 1) * FONT_HEIGHT][x * FONT_WIDTH] = char_pixel;
                }
            }
            // Clear the last row
            self.clear_row(FRAMEBUFFER_HEIGHT / FONT_HEIGHT - 1);
        }
    }

    fn clear_row(&mut self, row: usize) {
        let base_y = row * FONT_HEIGHT;
        for y in base_y..base_y + FONT_HEIGHT {
            for x in 0..FRAMEBUFFER_WIDTH {
                self.framebuffer.pixels[y][x] = 0; // Clear to black
            }
        }
    }

    fn draw_char(&mut self, col: usize, row: usize, character: u8, color: ColorCode) {
        let glyph = FONT[character as usize];
        let base_x = col * FONT_WIDTH;
        let base_y = row * FONT_HEIGHT;

        for (y, row) in glyph.iter().enumerate() {
            for x in 0..FONT_WIDTH {
                let pixel_color = if (row >> (FONT_WIDTH - 1 - x)) & 1 == 1 {
                    color.0
                } else {
                    0 // Background color (black)
                };
                self.framebuffer.pixels[base_y + y][base_x + x] = pixel_color;
            }
        }
    }

    pub fn draw_cursor(&mut self) {
        let base_x = self.column_position * FONT_WIDTH;
        let base_y = self.row_position * FONT_HEIGHT;

        for x in base_x..base_x + FONT_WIDTH {
            let y = base_y + FONT_HEIGHT - 1; // Underline position
            self.framebuffer.pixels[y][x] = self.cursor_color.0;
        }
    }

    pub fn erase_cursor(&mut self) {
        self.draw_char(self.column_position, self.row_position, b' ', self.color_code);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the framebuffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the framebuffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

/// Font data (8x16 pixels per character)
const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;
const FONT: [[u8; FONT_HEIGHT]; 256] = [
    // Define your font data here
    [0x00; FONT_HEIGHT]; 256
];
