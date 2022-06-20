#![no_std]
#![no_main]

use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    unsafe { qemu_shutdown(0x10_0000 as _, 0) }
}

/// Use the "SiFive Test Finisher" device to signal QEMU to terminate simulation
unsafe fn qemu_shutdown(test_finisher_device_ptr: *mut u32, exit_code: u16) -> ! {
    test_finisher_device_ptr.write((exit_code as u32) << 16 | 0x3333);
    // Never reached, assuming the device is present and we have the right address
    loop {}
}
