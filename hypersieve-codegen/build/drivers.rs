#[derive(serde::Deserialize, Debug)]
pub struct DriverConfig {
    pub mappings: std::collections::HashMap<String, String>,
}

pub fn parse() -> DriverConfig {
    let toml_content = std::fs::read_to_string(concat!(crate::root_dir!(), "drivers.toml"))
        .expect("[FATAL][BUILD]: failed to read the drivers description file");

    let config: DriverConfig =
        toml::from_str(&toml_content).expect("[FATAL][BUILD]: failed to parse Toml file");

    config
}
