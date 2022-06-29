use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use fdt::Fdt;
use portable_atomic::AtomicPtr;

pub fn parse(fdt_address: usize) -> Fdt<'static> {
    unsafe { Fdt::from_ptr(fdt_address as *const u8).expect("Failed to parse FDT") }
}

pub struct Register(AtomicPtr<()>);

impl Register {
    pub const fn new() -> Self {
        Self(AtomicPtr::new(core::ptr::null_mut()))
    }

    pub fn find_compatible(&self, with: &str, fdt: &Fdt) {
        let ptr = find_compatible_ptr(with, fdt);
        self.0.store(ptr, Ordering::Release);
    }

    pub fn cast<T>(&self) -> Option<NonNull<T>> {
        NonNull::new(self.0.load(Ordering::Acquire).cast())
    }
}

pub fn find_compatible_ptr<T>(with: &str, fdt: &Fdt) -> *mut T {
    try_find_compatible_ptr(with, fdt).expect("Failed to find device in FDT")
}

fn try_find_compatible_ptr<T>(with: &str, fdt: &Fdt) -> Option<*mut T> {
    let device = fdt.find_compatible(&[with])?;
    let register = device.reg()?.next()?;
    Some(register.starting_address as _)
}
