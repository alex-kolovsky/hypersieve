#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MmioPolicy {
    Emulate,
    Passthrough,
}
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base_addr: u64,
    pub size: u64,
}

impl MemoryRegion {
    pub fn end_addr(&self) -> u64 {
        self.base_addr + self.size - 1
    }

    pub fn is_aligned(&self) -> bool {
        (self.size & 4095 == 0) && (self.base_addr & 4095 == 0)
    }

    pub fn empty() -> Self {
        MemoryRegion {
            base_addr: 0x0,
            size: 0x0,
        }
    }
}
#[derive(Debug, Clone)]
pub struct MmioPeripheral {
    pub region: MemoryRegion,
    pub policy: MmioPolicy,
    pub compatible: String,
}

impl MmioPeripheral {
    pub fn is_emulate(&self) -> bool {
        self.policy == MmioPolicy::Emulate
    }
}

pub fn parse_dts(out_dir: &str) -> Vec<MmioPeripheral> {
    let dtb_output_path = std::path::PathBuf::from(out_dir).join("hardware.dtb");

    // Compile Device Tree file.
    crate::build_modules::main_dts_parser::compile_dts(&dtb_output_path, "hardware.dts");

    // Parse Device Tree Binary file.
    let dtb_data = std::fs::read(dtb_output_path).unwrap();

    let fdt = fdt::Fdt::new(&dtb_data).unwrap();

    let mut mmio_peripherals: Vec<MmioPeripheral> = Vec::new();

    if let Some(soc) = fdt.find_node("/soc") {
        for peripheral in soc.children() {
            let memory_policy_str = peripheral
                .property("memory-policy")
                .expect("No memory-policy field in the hardware DTS")
                .as_str()
                .unwrap();

            let policy = if memory_policy_str == "emulate" {
                MmioPolicy::Emulate
            } else if memory_policy_str == "passthrough" {
                MmioPolicy::Passthrough
            } else {
                panic!("Unsupported memory policy");
            };

            let compatible: String = peripheral
                .property("compatible")
                .expect("No compatible field in the hardware DTS")
                .as_str()
                .unwrap()
                .to_string();

            // Extract addresses of peripheral (reg).
            let mut regions: Vec<MmioPeripheral> = Vec::new();

            let reg = peripheral
                .property("reg")
                .expect("No reg field in the hardware DTS");
            // Convert a raw `reg` DTB property into native array.
            let reg_raw = reg.value;

            if reg_raw.len() % 8 != 0 {
                panic!("reg cell is not a multiple of 8 bytes");
            }

            let (chunks, _remainder) = reg_raw.as_chunks::<8>();

            for bytes in chunks.chunks(2) {
                let base_addr: u64 = u64::from_be_bytes(bytes[0]);
                let size: u64 = u64::from_be_bytes(bytes[1]);
                let region = MemoryRegion { base_addr, size };
                regions.push(MmioPeripheral {
                    region,
                    policy,
                    compatible: compatible.clone(),
                })
            }

            regions.sort_by_key(|memory_region| memory_region.region.base_addr);

            mmio_peripherals.extend(regions);
        }
    }
    mmio_peripherals.sort_by_key(|mmio_peripheral| mmio_peripheral.region.base_addr);
    mmio_peripherals
}
