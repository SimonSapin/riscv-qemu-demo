# Demo of freestanding Rust compiled to RISC-V running in QEMU

```
$ cargo test
    Finished dev [unoptimized + debuginfo] target(s) in 0.17s
     Running `qemu-system-riscv32 -nographic -machine virt -m 128 -bios none -kernel target/riscv32i-unknown-none-elf/debug/guest`
Hello from Rust RISC-V!
```
