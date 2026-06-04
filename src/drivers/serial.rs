use core::fmt;

use crate::arch::x86_64;

pub struct SerialConsole {
    port: x86_64::serial::SerialPort,
}

impl SerialConsole {
    pub fn new_com1() -> Self {
        Self {
            port: x86_64::serial::com1(),
        }
    }

    pub fn init(&mut self) {
        self.port.init();
    }
}

impl fmt::Write for SerialConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.port.write_str(s)
    }
}
