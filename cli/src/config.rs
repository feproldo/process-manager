use std::{env, fs, path::PathBuf, process};
use directories::ProjectDirs;

pub fn get_cfg() -> Result<serde_json::Value, serde_json::Error> {
    
    if let Some(proj_dirs) = ProjectDirs::from("com", "feproldo", "pm") {
        let config_dir: PathBuf = proj_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        let config_file = config_dir.join("config.json");
        let content = fs::read_to_string(&config_file).unwrap_or_else(|_| {
            if let Err(e) = fs::write(&config_file, "{}") {
                eprintln!("Failed to write to file: {}", e);
            }
            String::from("{}")
        });

        serde_json::from_str(&content)
    }
    else {
        process::exit(1);
    }
}

pub fn set_cfg (data: serde_json::Value) {
    if let Some(proj_dirs) = ProjectDirs::from("com", "feproldo", "pm") {
        let config_dir: PathBuf = proj_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        let config_file = config_dir.join("config.json");
        let file = fs::write(config_file, serde_json::to_string_pretty(&data).unwrap());
        match file {
            Ok(_) => {},
            Err(err) => {eprintln!("{err}"); process::exit(1)}
        }
    }
}