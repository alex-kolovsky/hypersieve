#[derive(serde::Deserialize, Debug)]
pub struct DriverConfig {
    pub mappings: std::collections::HashMap<String, String>,
}

pub fn parse() -> DriverConfig {
    let toml_content = std::fs::read_to_string("src/mmio/drivers.toml").unwrap();
    let config: DriverConfig = toml::from_str(&toml_content).unwrap();
    config
}
