use core::{fmt, ptr};

use crate::boot::FramebufferInfo;

const GLYPH_WIDTH: usize = 8;
const GLYPH_HEIGHT: usize = 8;

pub struct FramebufferConsole {
    info: FramebufferInfo,
    cursor_x: usize,
    cursor_y: usize,
    foreground: u32,
    background: u32,
}

impl FramebufferConsole {
    pub fn new(info: FramebufferInfo) -> Self {
        Self {
            info,
            cursor_x: 2,
            cursor_y: 2,
            foreground: 0x00e6edf3,
            background: 0x00101820,
        }
    }

    pub fn clear(&mut self) {
        if self.info.bits_per_pixel != 32 {
            return;
        }

        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.write_pixel(x, y, self.background);
            }
        }
    }

    pub fn draw_banner(&mut self) {
        if self.info.bits_per_pixel != 32 {
            return;
        }

        let height = self.info.height.min(96);
        for y in 0..height {
            let color = if y < 8 {
                0x0000b4d8
            } else if y < 16 {
                0x00f5c542
            } else {
                self.background
            };
            for x in 0..self.info.width {
                self.write_pixel(x, y, color);
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.cursor_x = 2,
            byte => {
                if self.cursor_x + GLYPH_WIDTH >= self.info.width {
                    self.new_line();
                }
                self.draw_char(byte);
                self.cursor_x += GLYPH_WIDTH;
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_x = 2;
        self.cursor_y += GLYPH_HEIGHT + 2;
    }

    fn draw_char(&mut self, byte: u8) {
        let glyph = glyph(byte);
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..GLYPH_WIDTH {
                let mask = 1 << (7 - col);
                let color = if bits & mask != 0 {
                    self.foreground
                } else {
                    self.background
                };
                self.write_pixel(self.cursor_x + col, self.cursor_y + row, color);
            }
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.info.width || y >= self.info.height || self.info.bits_per_pixel != 32 {
            return;
        }

        let offset = y * self.info.pitch + x * 4;
        unsafe {
            ptr::write_volatile(self.info.address.add(offset).cast::<u32>(), color);
        }
    }
}

impl fmt::Write for FramebufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

fn glyph(byte: u8) -> [u8; 8] {
    match byte {
        b' ' => [0; 8],
        b'-' => [0, 0, 0, 0b0111_1110, 0, 0, 0, 0],
        b'.' => [0, 0, 0, 0, 0, 0, 0b0001_1000, 0],
        b':' => [0, 0b0001_1000, 0b0001_1000, 0, 0, 0b0001_1000, 0b0001_1000, 0],
        b'0' => [0b0011_1100, 0b0110_0110, 0b0110_1110, 0b0111_0110, 0b0110_0110, 0b0110_0110, 0b0011_1100, 0],
        b'1' => [0b0001_1000, 0b0011_1000, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0111_1110, 0],
        b'2' => [0b0011_1100, 0b0110_0110, 0b0000_0110, 0b0000_1100, 0b0011_0000, 0b0110_0000, 0b0111_1110, 0],
        b'3' => [0b0011_1100, 0b0110_0110, 0b0000_0110, 0b0001_1100, 0b0000_0110, 0b0110_0110, 0b0011_1100, 0],
        b'4' => [0b0000_1100, 0b0001_1100, 0b0010_1100, 0b0100_1100, 0b0111_1110, 0b0000_1100, 0b0000_1100, 0],
        b'5' => [0b0111_1110, 0b0110_0000, 0b0111_1100, 0b0000_0110, 0b0000_0110, 0b0110_0110, 0b0011_1100, 0],
        b'6' => [0b0011_1100, 0b0110_0110, 0b0110_0000, 0b0111_1100, 0b0110_0110, 0b0110_0110, 0b0011_1100, 0],
        b'7' => [0b0111_1110, 0b0000_0110, 0b0000_1100, 0b0001_1000, 0b0011_0000, 0b0011_0000, 0b0011_0000, 0],
        b'8' => [0b0011_1100, 0b0110_0110, 0b0110_0110, 0b0011_1100, 0b0110_0110, 0b0110_0110, 0b0011_1100, 0],
        b'9' => [0b0011_1100, 0b0110_0110, 0b0110_0110, 0b0011_1110, 0b0000_0110, 0b0110_0110, 0b0011_1100, 0],
        b'A' | b'a' => [0b0001_1000, 0b0011_1100, 0b0110_0110, 0b0110_0110, 0b0111_1110, 0b0110_0110, 0b0110_0110, 0],
        b'B' | b'b' => [0b0111_1100, 0b0110_0110, 0b0110_0110, 0b0111_1100, 0b0110_0110, 0b0110_0110, 0b0111_1100, 0],
        b'C' | b'c' => [0b0011_1100, 0b0110_0110, 0b0110_0000, 0b0110_0000, 0b0110_0000, 0b0110_0110, 0b0011_1100, 0],
        b'D' | b'd' => [0b0111_1000, 0b0110_1100, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0110_1100, 0b0111_1000, 0],
        b'E' | b'e' => [0b0111_1110, 0b0110_0000, 0b0110_0000, 0b0111_1100, 0b0110_0000, 0b0110_0000, 0b0111_1110, 0],
        b'F' | b'f' => [0b0111_1110, 0b0110_0000, 0b0110_0000, 0b0111_1100, 0b0110_0000, 0b0110_0000, 0b0110_0000, 0],
        b'G' | b'g' => [0b0011_1100, 0b0110_0110, 0b0110_0000, 0b0110_1110, 0b0110_0110, 0b0110_0110, 0b0011_1110, 0],
        b'H' | b'h' => [0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0111_1110, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0],
        b'I' | b'i' => [0b0011_1100, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0011_1100, 0],
        b'K' | b'k' => [0b0110_0110, 0b0110_1100, 0b0111_1000, 0b0111_0000, 0b0111_1000, 0b0110_1100, 0b0110_0110, 0],
        b'L' | b'l' => [0b0110_0000, 0b0110_0000, 0b0110_0000, 0b0110_0000, 0b0110_0000, 0b0110_0000, 0b0111_1110, 0],
        b'M' | b'm' => [0b0110_0011, 0b0111_0111, 0b0111_1111, 0b0110_1011, 0b0110_0011, 0b0110_0011, 0b0110_0011, 0],
        b'N' | b'n' => [0b0110_0110, 0b0111_0110, 0b0111_1110, 0b0111_1110, 0b0110_1110, 0b0110_0110, 0b0110_0110, 0],
        b'O' | b'o' => [0b0011_1100, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0011_1100, 0],
        b'P' | b'p' => [0b0111_1100, 0b0110_0110, 0b0110_0110, 0b0111_1100, 0b0110_0000, 0b0110_0000, 0b0110_0000, 0],
        b'R' | b'r' => [0b0111_1100, 0b0110_0110, 0b0110_0110, 0b0111_1100, 0b0111_1000, 0b0110_1100, 0b0110_0110, 0],
        b'S' | b's' => [0b0011_1110, 0b0110_0000, 0b0110_0000, 0b0011_1100, 0b0000_0110, 0b0000_0110, 0b0111_1100, 0],
        b'T' | b't' => [0b0111_1110, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0b0001_1000, 0],
        b'U' | b'u' => [0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0011_1100, 0],
        b'V' | b'v' => [0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0110_0110, 0b0011_1100, 0b0011_1100, 0b0001_1000, 0],
        b'X' | b'x' => [0b0110_0110, 0b0110_0110, 0b0011_1100, 0b0001_1000, 0b0011_1100, 0b0110_0110, 0b0110_0110, 0],
        _ => [0b0111_1110, 0b0100_0010, 0b0101_1010, 0b0101_1010, 0b0101_1010, 0b0100_0010, 0b0111_1110, 0],
    }
}
