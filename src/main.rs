#![feature(sync_unsafe_cell)]
#![no_std]
#![no_main]

use core::cell::SyncUnsafeCell;
use core::fmt::{self, Write};
use core::ptr;
use fdt::Fdt;

static UART_PTR: SyncUnsafeCell<*mut u32> = SyncUnsafeCell::new(ptr::null_mut());
static TEST_FINISHER_PTR: SyncUnsafeCell<*mut u32> = SyncUnsafeCell::new(ptr::null_mut());

#[riscv_rt::entry]
fn main() -> ! {
    unsafe {
        let fdt = &Fdt::from_ptr(device_tree_ptr() as _).unwrap();
        find_register(fdt, "ns16550a", &UART_PTR).unwrap();
        find_register(fdt, "sifive,test0", &TEST_FINISHER_PTR).unwrap();
        let mut uart = Uart(UART_PTR.get().read());
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

unsafe fn find_register<T>(
    fdt: &Fdt,
    compatible_with: &str,
    cell: &SyncUnsafeCell<*mut T>,
) -> Option<*mut T> {
    let device = fdt.find_compatible(&[compatible_with])?;
    let register = device.reg()?.next()?;
    let ptr = register.starting_address as _;
    cell.get().write(ptr);
    Some(ptr)
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
        let ptr = UART_PTR.get().read();
        if !ptr.is_null() {
            let mut uart = Uart(ptr);
            let _ = writeln!(uart, "Rust panic: {}", info);
        }
        qemu_shutdown(1)
    }
}

/// Signal QEMU to terminate simulation, through the "SiFive Test Finisher" device
unsafe fn qemu_shutdown(exit_code: u16) -> ! {
    let ptr = TEST_FINISHER_PTR.get().read();
    if !ptr.is_null() {
        ptr.write((exit_code as u32) << 16 | 0x3333)
    }
    // Never reached, assuming the device is present and we have the right address
    loop {}
}
