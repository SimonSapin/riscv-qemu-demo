use crate::device_tree::Register;
use core::fmt;
use core::ptr::NonNull;

pub static REGISTER: Register = Register::new();

pub struct Uart(NonNull<u8>);

impl Uart {
    pub fn new() -> Option<Self> {
        unsafe { REGISTER.cast().map(Self) }
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe { self.0.as_ptr().write_volatile(byte) }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        s.bytes().for_each(|b| self.write_byte(b));
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($args: tt)*) => {
        if let Some(uart) = &mut $crate::uart::Uart::new() {
            use core::fmt::Write;
            let _ = writeln!(uart, $($args)*);
        }
    };
}
