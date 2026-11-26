use crate::build_modules::hardware_dts_parser::{MemoryRegion, MmioPeripheral};

pub fn partition_passthrough_mmio_regions(
    // All MMIO peripherals in the system.
    mmio_peripherals: &[crate::build_modules::hardware_dts_parser::MmioPeripheral],
    // All MMIO regions allowed to access by the guest in the system.
    allowed_regions: &[MemoryRegion],
) -> (Vec<MemoryRegion>, Vec<MmioPeripheral>) {
    let mut allowed_aligned_passthrough_mmio_addrs: Vec<MemoryRegion> = Vec::new();
    let mut allowed_peripherals: Vec<MmioPeripheral> = Vec::new();

    let mut i = 0;
    let mut p = 0;

    while p < mmio_peripherals.len() && i < allowed_regions.len() {
        let peripheral = &mmio_peripherals[p];
        let peripheral_region = peripheral.region;
        let allowed_region = &allowed_regions[i];

        let memory_region: MemoryRegion = if allowed_region.base_addr < peripheral_region.base_addr
            && allowed_region.end_addr() > peripheral_region.end_addr()
        {
            // The allowed region is larger than the peripheral region.
            //
            //              [ peripheral region ]
            //              |                   |
            //         [        allowed region         ]
            //              |                   |
            //              v                   v
            //              [intersecting  slice]
            //
            // Limit access strictly to the peripheral region to avoid accessing invalid memory.
            let base_addr = peripheral_region.base_addr;
            let size = peripheral_region.size;

            MemoryRegion { base_addr, size }
        } else if allowed_region.base_addr < peripheral_region.base_addr
            && allowed_region.end_addr() >= peripheral_region.base_addr
        {
            // The allowed region starts before the peripheral region but partially overlaps it.
            //
            //              [             peripheral region             ]
            //              |                  |
            //  [        allowed region        ]
            //              |                  |
            //              v                  v
            //              [intersecting slice]
            //
            // Clip the lower boundary to match the peripheral's base address.
            let base_addr = peripheral_region.base_addr;
            let end_addr = allowed_region.end_addr();
            let size = end_addr - base_addr + 1;

            MemoryRegion { base_addr, size }
        } else if allowed_region.base_addr >= peripheral_region.base_addr
            && allowed_region.end_addr() <= peripheral_region.end_addr()
        {
            // The allowed region either perfectly fits the peripheral region or lies entirely within it.
            //
            //             [   peripheral region    ]
            //                |                  |
            //                [  allowed region  ]
            //                |                  |
            //                v                  v
            //                [intersecting slice]
            //
            // Keep the allowed region as-is.
            let base_addr = allowed_region.base_addr;
            let size = allowed_region.size;

            MemoryRegion { base_addr, size }
        } else if allowed_region.base_addr <= peripheral_region.end_addr()
            && allowed_region.end_addr() > peripheral_region.end_addr()
        {
            // The allowed region ends after the peripheral region but partially overlaps it.
            //
            //    [             peripheral region             ]
            //                             |                  |
            //                             [            allowed region            ]
            //                             |                  |
            //                             v                  v
            //                             [intersecting slice]
            //
            // Clip the higher boundary to match the peripheral end address.
            let base_addr = allowed_region.base_addr;
            let end_addr = peripheral_region.end_addr();
            let size = end_addr - base_addr + 1;

            MemoryRegion { base_addr, size }
        } else if allowed_region.base_addr > peripheral_region.end_addr() {
            // The allowed region is strictly higher than the peripheral region.
            //
            //  [ peripheral region ]
            //
            //                             [  allowed region  ]
            //
            // Move to the next peripheral region.
            p += 1;

            continue;
        } else if allowed_region.end_addr() < peripheral_region.base_addr {
            // The allowed region is strictly lower than the peripheral region.
            //
            //                             [ peripheral region ]
            //
            //  [  allowed region  ]
            //
            // Move to the next allowed region.
            i += 1;

            continue;
        } else {
            unreachable!();
        };

        // If the address is passthrough and page-aligned, stage it for GPT entry generation.
        // Otherwise, route it to the fallback emulation vector.
        if !peripheral.is_emulate() && memory_region.is_aligned() {
            allowed_aligned_passthrough_mmio_addrs.push(memory_region);
        } else {
            allowed_peripherals.push(MmioPeripheral {
                region: memory_region,
                policy: peripheral.policy,
                compatible: peripheral.compatible.clone(),
            });
        }

        // If part of the allowed region exceeds the peripheral region, move to the next peripheral.
        // Otherwise, move to the next allowed region.
        if allowed_region.end_addr() > peripheral_region.end_addr() {
            p += 1;
        } else {
            i += 1;
        }
    }
    (allowed_aligned_passthrough_mmio_addrs, allowed_peripherals)
}
