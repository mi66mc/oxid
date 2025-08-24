pub struct String {
    buf: [u8; 64],
    len: usize,
}

impl String {
    pub const fn new() -> Self {
        Self {
            buf: [0; 64],
            len: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let mut string = Self::new();
        string.push_str(s);
        string
    }

    pub fn push_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let space = self.buf.len() - self.len;
        let to_copy = if bytes.len() > space { space } else { bytes.len() };
        self.buf[self.len..self.len+to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut new_str = Self::new();
        new_str.push_bytes(&self.buf[..self.len]);
        new_str.push_bytes(&other.buf[..other.len]);
        new_str
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        let space = self.buf.len() - self.len;
        let to_copy = if bytes.len() > space { space } else { bytes.len() };
        self.buf[self.len..self.len+to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}