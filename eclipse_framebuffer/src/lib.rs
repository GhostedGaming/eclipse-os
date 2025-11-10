#![no_std]

use core::fmt;
use core::cell::UnsafeCell;

#[repr(C, packed)]
struct PSF1Header {
    magic: [u8; 2],
    mode: u8,
    charsize: u8,
}

#[repr(C, packed)]
struct PSF2Header {
    magic: [u8; 4],
    version: u32,
    headersize: u32,
    flags: u32,
    numglyph: u32,
    bytesperglyph: u32,
    height: u32,
    width: u32,
}

struct RendererCell {
    inner: UnsafeCell<Option<ScrollingTextRenderer>>,
}

unsafe impl Sync for RendererCell {}

impl RendererCell {
    const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    fn set(&self, renderer: ScrollingTextRenderer) {
        unsafe {
            *self.inner.get() = Some(renderer);
        }
    }

    fn get(&self) -> &mut ScrollingTextRenderer {
        unsafe {
            (*self.inner.get())
                .as_mut()
                .expect("Renderer not initialized")
        }
    }
}

static RENDERER: RendererCell = RendererCell::new();

pub struct ScrollingTextRenderer {
    framebuffer: *mut u8,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: usize,
    x: usize,
    y: usize,
    fg_color: u32,
    bg_color: u32,
    font_data: &'static [u8],
    char_width: usize,
    char_height: usize,
    bytes_per_glyph: usize,
}

unsafe impl Send for ScrollingTextRenderer {}
unsafe impl Sync for ScrollingTextRenderer {}

impl ScrollingTextRenderer {
    pub fn init(
        framebuffer: *mut u8,
        width: usize,
        height: usize,
        pitch: usize,
        bpp: usize,
        font_data: &'static [u8],
    ) {
        let (char_width, char_height, bytes_per_glyph) = Self::parse_psf(font_data);
        
        let renderer = Self {
            framebuffer,
            width,
            height,
            pitch,
            bpp,
            x: 0,
            y: 0,
            fg_color: 0xFFFFFF,
            bg_color: 0x000000,
            font_data,
            char_width,
            char_height,
            bytes_per_glyph,
        };
        
        RENDERER.set(renderer);
    }

    pub fn get() -> &'static mut Self {
        RENDERER.get()
    }

    fn parse_psf(data: &[u8]) -> (usize, usize, usize) {
        if data.len() >= 32 && &data[0..4] == b"\x72\xb5\x4a\x86" {
            let header = unsafe { &*(data.as_ptr() as *const PSF2Header) };
            return (
                header.width as usize,
                header.height as usize,
                header.bytesperglyph as usize,
            );
        }
        
        if data.len() >= 4 && &data[0..2] == b"\x36\x04" {
            let header = unsafe { &*(data.as_ptr() as *const PSF1Header) };
            let height = header.charsize as usize;
            let width = 8;
            let bytes_per_glyph = height;
            return (width, height, bytes_per_glyph);
        }
        
        (8, 16, 16)
    }

    fn get_glyph_offset(&self, ch: char) -> usize {
        let idx = ch as usize;
        let max_glyphs = (self.font_data.len() - self.header_size()) / self.bytes_per_glyph;
        
        let glyph_idx = if idx < max_glyphs { idx } else { 0 };
        self.header_size() + glyph_idx * self.bytes_per_glyph
    }

    fn header_size(&self) -> usize {
        if self.font_data.len() >= 32 && &self.font_data[0..4] == b"\x72\xb5\x4a\x86" {
            let header = unsafe { &*(self.font_data.as_ptr() as *const PSF2Header) };
            header.headersize as usize
        } else {
            4
        }
    }

    pub fn set_colors(&mut self, fg: u32, bg: u32) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = y * self.pitch + x * (self.bpp / 8);
        unsafe {
            let pixel = self.framebuffer.add(offset) as *mut u32;
            *pixel = color;
        }
    }

    fn draw_char(&self, ch: char, x: usize, y: usize) {
        let glyph_offset = self.get_glyph_offset(ch);
        let glyph_data = &self.font_data[glyph_offset..glyph_offset + self.bytes_per_glyph];
        
        let bytes_per_line = (self.char_width + 7) / 8;
        
        for row in 0..self.char_height {
            let line_offset = row * bytes_per_line;
            
            for col in 0..self.char_width {
                let byte_idx = line_offset + (col / 8);
                let bit_idx = 7 - (col % 8);
                
                if byte_idx < glyph_data.len() {
                    let bit = (glyph_data[byte_idx] >> bit_idx) & 1;
                    let color = if bit == 1 { self.fg_color } else { self.bg_color };
                    self.put_pixel(x + col, y + row, color);
                }
            }
        }
    }

    fn scroll(&mut self) {
        let line_height = self.char_height;
        let bytes_per_pixel = self.bpp / 8;
        
        unsafe {
            for y in line_height..self.height {
                for x in 0..self.width {
                    let src_offset = y * self.pitch + x * bytes_per_pixel;
                    let dst_offset = (y - line_height) * self.pitch + x * bytes_per_pixel;
                    
                    let src = self.framebuffer.add(src_offset) as *const u32;
                    let dst = self.framebuffer.add(dst_offset) as *mut u32;
                    *dst = *src;
                }
            }
            
            let start_y = self.height - line_height;
            for y in start_y..self.height {
                for x in 0..self.width {
                    self.put_pixel(x, y, self.bg_color);
                }
            }
        }
        
        self.y -= line_height;
    }

    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.x = 0;
                self.y += self.char_height;
            }
            '\r' => {
                self.x = 0;
            }
            '\t' => {
                let tab_width = self.char_width * 4;
                self.x = ((self.x + tab_width) / tab_width) * tab_width;
                if self.x >= self.width {
                    self.x = 0;
                    self.y += self.char_height;
                }
            }
            _ => {
                if self.x + self.char_width > self.width {
                    self.x = 0;
                    self.y += self.char_height;
                }
                
                if self.y + self.char_height > self.height {
                    self.scroll();
                }
                
                self.draw_char(ch, self.x, self.y);
                self.x += self.char_width;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, self.bg_color);
            }
        }
        self.x = 0;
        self.y = 0;
    }

    pub fn panic_print(&mut self, s: &str) {
        self.clear();
        let center_y = self.height / 2;
        
        let line_count = s.lines().count();
        let total_text_height = line_count * self.char_height;
        
        let start_y = if center_y > total_text_height / 2 {
            center_y - total_text_height / 2
        } else {
            0
        };
        
        self.y = start_y;
        
        for line in s.lines() {
            let line_width: usize = line.chars().count() * self.char_width;
            let center_x = if self.width > line_width {
                (self.width - line_width) / 2
            } else {
                0
            };
            
            self.x = center_x;
            
            for ch in line.chars() {
                self.draw_char(ch, self.x, self.y);
                self.x += self.char_width;
            }
            
            self.x = 0;
            self.y += self.char_height;
        }
    }

    pub fn panic_write_str(&mut self, s: &str) {
        self.panic_print(s);
    }
}

impl fmt::Write for ScrollingTextRenderer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::ScrollingTextRenderer::get(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::ScrollingTextRenderer::get().write_char('\n')
    };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::ScrollingTextRenderer::get(), $($arg)*);
        $crate::ScrollingTextRenderer::get().write_char('\n');
    }};
}

#[macro_export]
macro_rules! panic_print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        
        struct StackString {
            buffer: [u8; 2048],
            len: usize,
        }
        
        impl StackString {
            fn new() -> Self {
                Self {
                    buffer: [0u8; 2048],
                    len: 0,
                }
            }
            
            fn as_str(&self) -> &str {
                core::str::from_utf8(&self.buffer[..self.len]).unwrap_or("")
            }
        }
        
        impl core::fmt::Write for StackString {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let remaining = self.buffer.len() - self.len;
                let to_write = core::cmp::min(bytes.len(), remaining);
                
                self.buffer[self.len..self.len + to_write].copy_from_slice(&bytes[..to_write]);
                self.len += to_write;
                
                Ok(())
            }
        }
        
        let mut buffer = StackString::new();
        let _ = write!(&mut buffer, $($arg)*);
        $crate::ScrollingTextRenderer::get().panic_write_str(buffer.as_str());
    }};
}