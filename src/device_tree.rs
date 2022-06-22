use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use fdt::Fdt;
use portable_atomic::AtomicPtr;

pub fn parse() -> Fdt<'static> {
    unsafe { Fdt::from_ptr(device_tree_ptr()).expect("Failed to parse FDT") }
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

pub struct Register(AtomicPtr<()>);

impl Register {
    pub const fn new() -> Self {
        Self(AtomicPtr::new(core::ptr::null_mut()))
    }

    pub fn find_compatible(&self, with: &str, fdt: &Fdt) {
        self.try_find_compatible(with, fdt)
            .expect("Failed to find device in FDT");
    }

    fn try_find_compatible(&self, with: &str, fdt: &Fdt) -> Option<()> {
        let device = fdt.find_compatible(&[with])?;
        let register = device.reg()?.next()?;
        let ptr = register.starting_address as _;
        self.0.store(ptr, Ordering::SeqCst);
        Some(())
    }

    pub fn cast<T>(&self) -> Option<NonNull<T>> {
        NonNull::new(self.0.load(Ordering::SeqCst).cast())
    }
}
