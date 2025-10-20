use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

const LIN_CONFIG_LOC: &str = "~/.config/zsmm/";
const OS: &str = std::env::consts::OS;

pub async fn check_config_dir() {
    let directory = match OS {
        "linux" => LIN_CONFIG_LOC,
        _ => "",
    };

    match fs::read_dir(directory).await {
        Ok(_entry) => {}
        Err(_err) => mk_config().await,
    }
}

pub async fn mk_config() {
    let _ = match OS {
        "linux" => Command::new(format!("mkdir {}", LIN_CONFIG_LOC)),
        _ => Command::new("echo error identifying OS"),
    };
}

pub async fn write_config(
    file_name: &str,
    mods_directory: &str,
    selections: HashMap<String, bool>,
) {
    let mut output: String = String::new();
    let config_file = OS.to_owned() + file_name;
    output.push_str(&format!("dir: {}\n", mods_directory));
    output.push_str("MapValues: ");

    for (id, bool) in selections {
        let insertion = format!("<{},{}>", id, bool);

        output.push_str(&insertion);
    }

    let _ = fs::write(config_file, output).await;
}

pub async fn read_config(file_name: &str) -> (&str, HashMap<String, bool>) {
    let config_path = OS.to_owned() + file_name;
    
    todo!()
}
