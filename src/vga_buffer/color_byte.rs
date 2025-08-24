#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: u8, background: u8) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}