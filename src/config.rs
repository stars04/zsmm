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
pub async fn write_selection_config(
    file_name: String,
    selections: HashMap<String, bool>,
    mod_ids: Vec<String>,
) {
    let mut output: String = String::new();

    let config_file = LIN_CONFIG_LOC.to_owned() + &file_name;

    for (name, bool) in selections {
        output.push_str(&format!("{},{};", name, bool));
    }

    output.push('\n');

    for id in mod_ids {
        output.push_str(&format!("{};",&id));
    } 
    

    let _ = fs::write(config_file, output).await;
}

//TODO: CLEAN THIS TF UP
pub async fn read_config(file_name: String) -> (Vec<String>, HashMap<String, bool>) {
    let mut output_map: HashMap<String, bool> = HashMap::new();
    let mut mod_id_vec: Vec<String> = Vec::new();
    let mut buffer: Vec<u8> = Vec::new();

    let config_path: String = file_name;
    let mut file = match File::open(&config_path).await {
        Ok(file) => file,
        Err(err) => panic!("Error reading {} -> Err: {}", config_path, err),
    };
    let _ = file.read_to_end(&mut buffer).await;

    if let Ok(text) = str::from_utf8(&buffer) {
        let mut string: String = text.to_string();
        println!("{:?}", string);
        let mut inspection: String;
        let mut chop: usize;
        let mut comma_loc: usize;
        let mut key: String;
        let mut value: String;

        //string = string.replace("\n", "");
        while !string.is_empty() {
            let ret_location: Option<usize> = string.find("\n");
            inspection = string.clone();
            chop = inspection.find(';').unwrap();
            inspection.replace_range(chop.., "");
            string.replace_range(..(chop + 1), "");

            if ret_location.is_some() && ret_location != Some(0_usize) {
                comma_loc = inspection.find(',').unwrap();
                value = inspection.split_off(comma_loc).replace(",", "");
                key = inspection;

                output_map.insert(key, value.parse::<bool>().unwrap());
            }else if ret_location == Some(0) {
                inspection = inspection.replace("\n", "");
                string.replace_range(..ret_location.unwrap(), "");

                mod_id_vec.push(inspection);
            }else if ret_location.is_none() {
                mod_id_vec.push(inspection);
            }
        }
        
    }

    (mod_id_vec, output_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn testing() {
        let result = read_config("/home/star/.config/zsmm/test".to_string()).await;
        println!("\n\n{:?}", result);
        //let result = write_selection_config("initial".to_string(), "/mnt/d1/SSD1/steamapps/workshop/content/108600/".to_string(),HashMap::new()).await;
                
    }
}
