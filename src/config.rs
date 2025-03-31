use config::{Config, File};
use std::fs;

pub fn get_config() -> Config {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))
        .unwrap_or_else(|_| {
            println!("Could not load configuration file");
            std::process::exit(1);
        })
        .join(get_program_name());

    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir).unwrap_or_else(|_| {
        println!("Could not create configuration directory");
        std::process::exit(1);
    });

    let config_path = config_dir.join("config.toml");

    // Create default config if it doesn't exist
    if !config_path.exists() {
        let default_config = r#"# Default configuration for apisnip
[default]
# Enable verbose output
verbose = false
"#;
        fs::write(&config_path, default_config).unwrap_or_else(|_| {
            println!("Could not create default configuration file");
            std::process::exit(1);
        });
    }

    let config = Config::builder()
        .add_source(File::from(config_path).required(true))
        .build()
        .unwrap_or_else(|_| {
            println!("Could not load configuration file");
            std::process::exit(1);
        });

    config
}

fn get_program_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|exe_path| {
            exe_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| {
            println!("Failed to get executable path");
            "unknown".to_string()
        })
}
