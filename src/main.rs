#![no_std]
#![no_main]

mod device_tree;
#[macro_use]
mod uart;
mod ramfb;
mod test_finisher;

#[riscv_rt::entry]
fn main(_hartid: usize, fdt_address: usize) -> ! {
    unsafe {
        let fdt = &device_tree::parse(fdt_address);
        uart::REGISTER.find_compatible("ns16550a", fdt);
        test_finisher::REGISTER.find_compatible("sifive,test0", fdt);

        println!("Hello from Rust RISC-V!");

        let fw_cfg = device_tree::find_compatible_ptr("qemu,fw-cfg-mmio", fdt);
        let mut fw_cfg = qemu_fw_cfg::FwCfg::new_memory_mapped(fw_cfg).unwrap();

        let pixel_format = ramfb::PixelFormat::B8G8R8;
        let config = ramfb::RamFbConfig {
            pixel_format,
            width_pixels: 100,
            height_pixels: 100,
            stride_bytes: 300,
        };
        let buffer = [[0xff_u8, 0x88, 0]; 100 * 100];
        ramfb::configure(&mut fw_cfg, &config, &buffer).unwrap();

        if cfg!(not(test)) {
            println!("Echoing. CTRL+C or CTRL+D to exit");
            let uart = uart::Uart::new().unwrap();
            loop {
                if let Some(b) = uart.read_byte() {
                    if b == b'\x03' || b == b'\x04' {
                        break;
                    }
                    uart.write_byte(b);
                    if b == b'\r' {
                        uart.write_byte(b'\n')
                    }
                }
            }
        }

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
