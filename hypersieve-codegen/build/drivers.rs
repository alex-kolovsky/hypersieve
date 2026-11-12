#[derive(serde::Deserialize, Debug)]
pub struct DriverConfig {
    pub mappings: std::collections::HashMap<String, String>,
}

pub fn parse(target_arch: &crate::TargetArch) -> DriverConfig {
    let toml_content = crate::build_modules::arch::get_driver_config(target_arch);
    let config: DriverConfig = toml::from_str(&toml_content).unwrap();
    config
}
