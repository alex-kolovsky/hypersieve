fn main() {
    println!("cargo:rerun-if-changed=config.dts");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Compile Device Tree file
    let dtb_output_path = std::path::PathBuf::from(&out_dir).join("config.dtb");
    let dts_source_path = "config.dts";

    let dtc_output = std::process::Command::new("dtc")
        .arg("-I")
        .arg("dts")
        .arg("-O")
        .arg("dtb")
        .arg("-o")
        .arg(&dtb_output_path)
        .arg(dts_source_path)
        .output()
        .expect("Failed to execute 'dtc'. Is device-tree-compiler installed on your system?");

    if !dtc_output.status.success() {
        panic!(
            "DTC Compilation Error: {}",
            String::from_utf8_lossy(&dtc_output.stderr)
        );
    }
}
