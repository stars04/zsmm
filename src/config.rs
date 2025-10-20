use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::slice;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::config;

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

//TODO: CLEAN THIS TF UP
pub async fn read_config(file_name: &str) -> (String, HashMap<String, bool>) {
    let mut output_string: String = String::new();
    let mut output_map: HashMap<String, bool> = HashMap::new();
    let mut buffer: Vec<u8> = Vec::new();
    let config_path: String = LIN_CONFIG_LOC.to_owned() + file_name;
    let mut file = match File::open(&config_path).await {
        Ok(file) => file,
        Err(err) => panic!("Error reading {} -> Err: {}", config_path, err),
    };
    let _ = file.read_to_end(&mut buffer).await;

    if let Ok(text) = str::from_utf8(&buffer) {
        let mut string = text.to_string();
        let initial_offset = string.find(':').unwrap() + 2_usize;
        string = string.split_off(initial_offset);

        let new_line = string.find('\n').unwrap();
        output_string = string.to_string().clone();
        output_string.replace_range(new_line.., "");
        string.replace_range(..(new_line + 1_usize), "");
        string = string.replace("\n", "");

        loop {
            if !string.is_empty() {
                let mut inspection = string.clone();
                let chop = inspection.find('>').unwrap();
                inspection.replace_range(chop.., "");
                string.replace_range(..(chop + 1), "");

                let mid = inspection.find(',').unwrap();
                let mut value = inspection.split_off(mid);
                let mut key = inspection;
                key = key.replace("<", "");
                value = value.replace(",", "");

                println!("======\n{:?}\n{:?}\n{:?}", key, value, string);
                output_map.insert(key, value.parse::<bool>().unwrap());
            } else {
                break;
            }
        }
    }

    (output_string, output_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn testing() {
        let result = read_config("test").await;
        println!("{:?}", result);
    }
}
