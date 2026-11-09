#!/bin/sh

# Create directories for guest binaries.
mkdir -p target/guest-1 target/guest-2

for i in 1 2; do
  # Compile guest files and extract binaries from .elf files.
  clang \
    -Wall -Wextra --target=riscv64-unknown-elf -march=rv64gcv \
    -ffreestanding -nostdlib -fuse-ld=lld -Wl,-eguest_boot \
    -Wl,-Ttext=0x100000 -Wl,-Map=target/guest-$i/guest-$i.map \
    src/arch/riscv64/guests/guest-$i.S -o target/guest-$i/guest-$i.elf && \

  llvm-objcopy -O binary target/guest-$i/guest-$i.elf target/guest-$i/guest-$i.bin
done
