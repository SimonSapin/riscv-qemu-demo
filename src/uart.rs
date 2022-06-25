use crate::device_tree::Register;
use core::fmt;
use core::ptr::NonNull;

pub static REGISTER: Register = Register::new();

pub struct Uart(NonNull<u8>);

impl Uart {
    pub fn new() -> Option<Self> {
        REGISTER.cast().map(Self)
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe { self.0.as_ptr().write_volatile(byte) }
    }

    pub fn read_byte(&self) -> Option<u8> {
        unsafe {
            let base_ptr = self.0.as_ptr();
            if base_ptr.add(5).read_volatile() & 1 != 0 {
                Some(base_ptr.read_volatile())
            } else {
                None
            }
        }
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

// Adapted from https://github.com/rust-lang/rust/blob/1.61.0/library/std/src/macros.rs#L299-L321
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        println!("[{}:{}]", core::file!(), core::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                println!("[{}:{}] {} = {:#?}",
                    core::file!(), core::line!(), core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
