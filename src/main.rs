#![no_std]
#![no_main]

use core::fmt::{self, Write};
use core::ptr;
use fdt::Fdt;

// FIXME: can we replace this with something Sync without atomic instructions?
static mut UART_PTR: *mut u32 = ptr::null_mut();
static mut TEST_FINISHER_PTR: *mut u32 = ptr::null_mut();

#[riscv_rt::entry]
fn main() -> ! {
    unsafe {
        let fdt = &Fdt::from_ptr(device_tree_ptr() as _).unwrap();
        UART_PTR = register_for_compatible_device(fdt, "ns16550a").unwrap();
        TEST_FINISHER_PTR = register_for_compatible_device(fdt, "sifive,test0").unwrap();
        let mut uart = Uart(UART_PTR);
        writeln!(uart, "Hello from Rust RISC-V!").unwrap();
        qemu_shutdown(0)
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

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let ptr = UART_PTR;
        if !ptr.is_null() {
            let mut uart = Uart(UART_PTR);
            let _ = writeln!(uart, "Rust panic: {}", info);
        }
        qemu_shutdown(1)
    }
}

/// Signal QEMU to terminate simulation, through the "SiFive Test Finisher" device
unsafe fn qemu_shutdown(exit_code: u16) -> ! {
    let ptr = TEST_FINISHER_PTR;
    if !ptr.is_null() {
        ptr.write((exit_code as u32) << 16 | 0x3333)
    }
    // Never reached, assuming the device is present and we have the right address
    loop {}
}
