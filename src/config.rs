use std::collections::HashMap;
use std::env::{consts, home_dir};
use std::path::Path;
use std::process::Command;
use std::slice;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::config;

pub const LIN_CONFIG_LOC: &str = "/home/star/.config/zsmm/";
const OS: &str = consts::OS;

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

pub async fn load_workshop_location() -> Option<String> {
    let mut output_string: String = String::new();
    let mut buffer: Vec<u8> = Vec::new();
    let config_path: String = LIN_CONFIG_LOC.to_owned() + "workshop_location";
    let mut file = match File::open(&config_path).await {
        Ok(file) => file,
        Err(err) => panic!("Error reading {} -> Err: {}", config_path, err),
    };
    let _ = file.read_to_end(&mut buffer).await;

    if let Ok(text) = str::from_utf8(&buffer) {

        let string: String = text.to_string();

        output_string = string.to_string().replace("\n", "");

    }
    Some(output_string)
}

pub async fn save_workshop_location(mods_directory: String) {
    let mut output: String = String::new();
    output.push_str(&mods_directory);
    
    let _ = fs::write(LIN_CONFIG_LOC.to_owned() + "workshop_location", output).await;
}
pub async fn write_config(
    file_name: String,
    selections: HashMap<String, bool>,
) {
    let mut output: String = String::new();
    let config_file = LIN_CONFIG_LOC.to_owned() + &file_name;


    for (id, bool) in selections {
        let insertion = format!("<{},{}>", id, bool);

        output.push_str(&insertion);
    }

    let _ = fs::write(config_file, output).await;
}

//TODO: CLEAN THIS TF UP
pub async fn read_config(file_name: &str) -> HashMap<String, bool> {
    let mut output_map: HashMap<String, bool> = HashMap::new();
    let mut buffer: Vec<u8> = Vec::new();
    let config_path: String = LIN_CONFIG_LOC.to_owned() + file_name;
    let mut file = match File::open(&config_path).await {
        Ok(file) => file,
        Err(err) => panic!("Error reading {} -> Err: {}", config_path, err),
    };
    let _ = file.read_to_end(&mut buffer).await;

    if let Ok(text) = str::from_utf8(&buffer) {
        let mut string: String = text.to_string();
        let mut inspection: String;
        let mut chop: usize;
        let mut comma_loc: usize;
        let mut key: String;
        let mut value: String;

        //let initial_offset = string.find(':').unwrap() + 2_usize;
        //string = string.split_off(initial_offset);

        //let new_line = string.find('\n').unwrap();

        //output_string = string.to_string().clone();
        //output_string.replace_range(new_line.., "");

        //string.replace_range(..(new_line + 1_usize), "");
        string = string.replace("\n", "");
        loop {
            if !string.is_empty() {
                inspection = string.clone();
                chop = inspection.find('>').unwrap();
                inspection.replace_range(chop.., "");
                string.replace_range(..(chop + 1), "");

                comma_loc = inspection.find(',').unwrap();
                value = inspection.split_off(comma_loc).replace(",", "");
                key = inspection.replace("<", "");

                output_map.insert(key, value.parse::<bool>().unwrap());
            } else {
                break;
            }
        }
    }

    output_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn testing() {
        let result = save_workshop_location("/mnt/d1/SSD1/steamapps/workshop/content/108600/".to_string()).await;
        //let result = write_config("initial".to_string(), "/mnt/d1/SSD1/steamapps/workshop/content/108600/".to_string(),HashMap::new()).await;
    }
}
