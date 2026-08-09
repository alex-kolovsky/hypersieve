# Hypersieve

A bare-metal type 1 Hypervisor written from scratch in Rust

## Getting Started

### Compiling a Test Guest File

To compile the guest assembly test file,
you'll need the [clang](https://clang.org/) compiler infrastructure and [llvm-objcopy](https://llvm.org/docs/CommandGuide/llvm-objcopy.html) installed on your host machine.

Execute this command inside the root directory of the project to build it:

```sh
clang -Wall -Wextra --target=riscv64-unknown-elf -march=rv64gcv -ffreestanding -nostdlib -fuse-ld=lld -Wl,-eguest_boot -Wl,-Ttext=0x100000 -Wl,-Map=guest.map guest.S -o guest.elf && llvm-objcopy -O binary guest.elf guest.bin
```

### Running the hypervisor

After compiling the guest binary, you need to compile and execute the core hypervisor.
To do this, the [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) build utility is required.

Run this to automatically compile and run the hypervisor:

```sh
cargo run
```
