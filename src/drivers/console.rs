use core::{cell::UnsafeCell, fmt};

use crate::{
    boot::FramebufferInfo,
    drivers::{framebuffer::FramebufferConsole, serial::SerialConsole},
};

pub struct Console {
    serial: SerialConsole,
    framebuffer: Option<FramebufferConsole>,
}

impl Console {
    pub fn new(framebuffer: Option<FramebufferInfo>) -> Self {
        let mut serial = SerialConsole::new_com1();
        serial.init();

        let mut framebuffer = framebuffer.map(FramebufferConsole::new);
        if let Some(framebuffer) = framebuffer.as_mut() {
            framebuffer.clear();
            framebuffer.draw_banner();
        }

        Self { serial, framebuffer }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial.write_str(s)?;
        if let Some(framebuffer) = self.framebuffer.as_mut() {
            framebuffer.write_str(s)?;
        }
        Ok(())
    }
}

struct GlobalConsole(UnsafeCell<Option<Console>>);

unsafe impl Sync for GlobalConsole {}

static CONSOLE: GlobalConsole = GlobalConsole(UnsafeCell::new(None));

pub fn init(framebuffer: Option<FramebufferInfo>) {
    unsafe {
        *CONSOLE.0.get() = Some(Console::new(framebuffer));
    }
}

pub fn write(args: fmt::Arguments) {
    use core::fmt::Write;

    unsafe {
        if let Some(console) = (*CONSOLE.0.get()).as_mut() {
            let _ = console.write_fmt(args);
        }
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    write(args);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::drivers::console::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! kprintln {
    () => {
        $crate::kprint!("\n")
    };
    ($($arg:tt)*) => {
        $crate::kprint!("{}\n", format_args!($($arg)*))
    };
}
