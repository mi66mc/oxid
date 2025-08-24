use crate::vga_buffer::color_byte::ColorCode;
use crate::vga_buffer::color::Color;
use core::ptr::{write_volatile};
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    char: u8,
    color: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_pos: usize,
    color: ColorCode,
    buffer: &'static mut Buffer
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let ptr = &mut self.buffer.chars[BUFFER_HEIGHT -1][self.column_pos] as *mut ScreenChar;
                unsafe {
                    write_volatile(ptr, ScreenChar {
                        char: byte,
                        color: self.color,
                    });
                }
                self.column_pos += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not in range
                _ => self.write_byte(0xfe),
            }

        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for column in 0..BUFFER_WIDTH {
                let ch = self.buffer.chars[row][column];
                self.buffer.chars[row - 1][column] = ch;
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            char: 0,
            color: self.color,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col] = blank;
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub static mut WRITER: Option<Writer> = None;

pub fn init_writer() {
    unsafe {
        WRITER = Some(Writer {
            column_pos: 0,
            color: ColorCode::new(Color::White as u8, Color::Red as u8),
            buffer: &mut *(0xb8000 as *mut Buffer),
        });
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe { let _ = WRITER.as_mut().unwrap().write_fmt(args); }
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => ($crate::vga_buffer::buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}