#![no_std]
#![no_main]

use core::fmt::{self, Write};
use fdt::Fdt;
use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    unsafe {
        let fdt = &Fdt::from_ptr(device_tree_ptr() as _).unwrap();
        let uart_ptr = register_for_compatible_device(fdt, "ns16550a").unwrap();
        let mut uart = Uart(uart_ptr);
        writeln!(uart, "Hello RISC-V!").unwrap();
        let test_finisher = register_for_compatible_device(fdt, "sifive,test0").unwrap();
        qemu_shutdown(test_finisher, 0)
    }
}

/// Pointer to the Flattened Device Tree
unsafe fn device_tree_ptr() -> *const u8 {
    // QEMU starts simulation with a boot ROM that puts the FDT pointer in register A1
    // before calling the firmware we provide: https://stackoverflow.com/a/72060395
    // However by the time Rust code is running,
    // startup code from the `riscv_rt` crate has already reset all registers to zero.
    //
    // As a work-around, we rely on the code for that boot ROM being
    // at a known address with a known structure to recover the FDT pointer.

    // https://gitlab.com/qemu-project/qemu/-/blob/v7.0.0/hw/riscv/virt.c#L72
    let boot_rom = 0x1000 as *mut u32;
    // https://gitlab.com/qemu-project/qemu/-/blob/v7.0.0/hw/riscv/boot.c#L297-317
    let offset = 8; // in 32-bit words

    boot_rom.add(offset).cast::<*const u8>().read()
}

fn register_for_compatible_device<T>(fdt: &Fdt, compatible_with: &str) -> Option<*mut T> {
    let device = fdt.find_compatible(&[compatible_with])?;
    let register = device.reg()?.next()?;
    Some(register.starting_address as _)
}

struct Uart(*mut u32);

impl Uart {
    fn write_byte(&self, byte: u8) {
        unsafe { self.0.write_volatile(byte as _) }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        s.bytes().for_each(|b| self.write_byte(b));
        Ok(())
    }
}

/// Signal QEMU to terminate simulation, through the "SiFive Test Finisher" device
unsafe fn qemu_shutdown(test_finisher_device_ptr: *mut u32, exit_code: u16) -> ! {
    test_finisher_device_ptr.write((exit_code as u32) << 16 | 0x3333);
    // Never reached, assuming the device is present and we have the right address
    loop {}
}
