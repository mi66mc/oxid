pub struct FixedString<const CAPACITY: usize> {
    bytes: [u8; CAPACITY],
    len: usize,
}

impl<const CAPACITY: usize> FixedString<CAPACITY> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; CAPACITY],
            len: 0,
        }
    }

    pub fn push_str(&mut self, value: &str) {
        self.push_bytes(value.as_bytes());
    }

    pub fn push_ascii_byte(&mut self, byte: u8) {
        if self.len < CAPACITY && byte.is_ascii() {
            self.bytes[self.len] = byte;
            self.len += 1;
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes.iter().copied().filter(u8::is_ascii) {
            self.push_ascii_byte(byte);
        }
    }
}

impl<const CAPACITY: usize> Default for FixedString<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::FixedString;

    #[test]
    fn appends_ascii_text() {
        let mut value = FixedString::<8>::new();

        value.push_str("Ox");
        value.push_str("id");

        assert_eq!(value.as_str(), "Oxid");
    }

    #[test]
    fn truncates_at_capacity() {
        let mut value = FixedString::<4>::new();

        value.push_str("Oxid Kernel");

        assert_eq!(value.as_str(), "Oxid");
    }

    #[test]
    fn drops_non_ascii_bytes() {
        let mut value = FixedString::<8>::new();

        value.push_str("Oxid \u{1f980}");

        assert_eq!(value.as_str(), "Oxid ");
    }
}
