#![no_std]
#![no_main]

mod device_tree;
#[macro_use]
mod uart;
mod test_finisher;

#[riscv_rt::entry]
fn main() -> ! {
    unsafe {
        let fdt = &device_tree::parse();
        uart::REGISTER.find_compatible("ns16550a", fdt);
        test_finisher::REGISTER.find_compatible("sifive,test0", fdt);

        println!("Hello from Rust RISC-V!");

        test_finisher::qemu_shutdown(0)
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        println!("Rust panic: {}", info);
        test_finisher::qemu_shutdown(1)
    }
}
