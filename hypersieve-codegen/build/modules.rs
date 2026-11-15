use std::io::Write;

#[derive(serde::Deserialize, Debug)]
pub struct ModulesConfig {
    // The mappings field represents a module's name along with its path.
    pub mappings: std::collections::HashMap<String, String>,
}

pub fn parse() -> ModulesConfig {
    // Parse the toml file directly into the ModulesConfig struct.
    let toml_content = std::fs::read_to_string(crate::MODULES_TOML_PATH).unwrap();
    let config: ModulesConfig = toml::from_str(&toml_content).unwrap();
    config
}

pub fn generate_module_paths(target_arch: &crate::TargetArch, out_dir: &str) {
    let module_mappings_file_path = std::path::Path::new(out_dir).join("module_mappings.rs");
    let mut module_mappings_file = std::fs::File::create(module_mappings_file_path).unwrap();
    let module_config = parse();

    // Ensure that all required modules are declared in the file.
    for req in target_arch.get_required_modules() {
        assert!(
            module_config.mappings.contains_key(&req),
            "Missing nodule declaration: {}",
            req
        );
    }

    // Generate static variables containing module data.
    for (module_name, relative_path) in module_config.mappings {
        let absolute_path = format!(
            "{}{}",
            crate::root_dir_from_out_relative!(),
            relative_path.as_str()
        );
        let str = format!(
            "pub static {}_MODULE_WASM: &[u8] = include_bytes!(\"{absolute_path}\");",
            module_name.to_uppercase()
        );
        writeln!(module_mappings_file, "{str}").unwrap();
    }
}
