
use ::errors::*;

use config::Config;
pub fn load_config_from_file(path: &str) -> Result<Config> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).chain_err(|| "Failed to open config file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).chain_err(|| "Failed to read file")?;

    let config = ::toml::de::from_str(&contents).chain_err(|| "Failed to deserialize config")?;

    Ok(config)
}

pub fn get_error_trace(e: &Error) -> String {
    let mut error_trace = String::new();
    error_trace.push_str("Error: ");
    error_trace.push_str(&e.to_string());
    for e in e.iter().skip(1) {
        error_trace.push_str("\nCause: ");
        error_trace.push_str(&e.to_string());
    }
    error_trace
}
