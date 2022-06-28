pub use crate::device_tree::Register;

pub static REGISTER: Register = Register::new();

/// Signal QEMU to terminate simulation, through the "SiFive Test Finisher" device
pub unsafe fn qemu_shutdown(exit_code: u16) -> ! {
    if let Some(ptr) = REGISTER.cast::<u32>() {
        ptr.as_ptr()
            .write_volatile((exit_code as u32) << 16 | 0x3333)
    }
    // Never reached, assuming the device is present and we have the right address
    loop {}
}
