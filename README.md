# Hypersieve

A bare-metal type 1 Hypervisor written from scratch in Rust

## Getting Started

### Compiling a Test Guest File

To compile the guest assembly test file,
you'll need the [clang](https://clang.org/) compiler infrastructure and [llvm-objcopy](https://llvm.org/docs/CommandGuide/llvm-objcopy.html) installed on your host machine.

Execute this command inside the root directory of the project to build it:

```sh
mkdir -p target/guest-{1,2} && for i in 1 2; do clang -Wall -Wextra --target=riscv64-unknown-elf -march=rv64gcv -ffreestanding -nostdlib -fuse-ld=lld -Wl,-eguest_boot -Wl,-Ttext=0x100000 -Wl,-Map=target/guest-$i/guest-$i.map src/asm/guest-$i.S -o target/guest-$i/guest-$i.elf && llvm-objcopy -O binary target/guest-$i/guest-$i.elf target/guest-$i/guest-$i.bin; done
```

### Running the hypervisor

After compiling the guest binary, you need to compile and execute the core hypervisor.
To do this, the [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) build utility is required.

Run this to automatically compile and run the hypervisor:

```sh
cargo run
```

## License
This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT License](https://opensource.org/license/MIT)

at your option.


<sub>Copyright (c) 2026 Alex Kolovsky. All rights reserved.</sub>
