# Moved to Codeberg

<https://codeberg.org/SimonSapin/riscv-qemu-demo>

## Demo of freestanding Rust compiled to RISC-V running in QEMU

```rust
println!("Hello from Rust RISC-V!");

let pixel_format = ramfb::PixelFormat::B8G8R8;
let config = ramfb::RamFbConfig {
    pixel_format,
    width_pixels: 100,
    height_pixels: 100,
    stride_bytes: 300,
};
let buffer = [[0xff_u8, 0x88, 0]; 100 * 100];
ramfb::configure(&mut fw_cfg, &config, &buffer).unwrap();
```

```
$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running `qemu-system-riscv32 -machine virt -m 128 -device ramfb -serial stdio -bios none -kernel target/riscv32i-unknown-none-elf/debug/guest`
Hello from Rust RISC-V!
```

<img src=qemu-gui.png width=372>
