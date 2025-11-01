use std::boxed::Box;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::str;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub enum Target {
    Id,
    Description,
    Name,
}

#[derive(Clone, PartialEq)]
pub enum FileType {
    ModInfo,
    Png,
}

pub async fn mod_file_finder(starting_dir: String, target_type: FileType) -> String { 
    let mut dir_vec: Vec<String> = Vec::new();
    let exit_val: String;
    if let Ok(mut entry) = fs::read_dir(&starting_dir).await {
        while let Ok(sub_entry) = entry.next_entry().await {
            if let Some(subdir) = &sub_entry && subdir.path().is_file() {
                    let possible_target = subdir.path().to_str().unwrap().to_string();
                    match target_type {
                        FileType::Png => {
                            if possible_target.contains(".png") {
                                return possible_target; 
                            }
                        },
                        FileType::ModInfo => {
                            if possible_target.contains("mod.info") {
                                return possible_target;
                            }
                        },
                    }
            } else if let Some(subdir) = &sub_entry && subdir.path().is_dir() {
                    dir_vec.push(subdir.path().to_str().unwrap().to_string())
            } else {
                break;
            } 
        }

        for path in &dir_vec {
            let return_val = Box::pin(mod_file_finder(path.to_string(), target_type.clone())).await;
            if !return_val.is_empty() {
                exit_val = return_val;
                return exit_val;
            }
        }
    }
    "".to_string()
}

//=== Function for getting ModIds =====
pub async fn id_path_process(input_vec: Vec<String>) -> std::io::Result<Vec<String>> {
    let mut output: Vec<String> = Vec::new();
    for info_file in input_vec {
        if let Ok(info) = mod_info_parse(info_file, Target::Id).await {
            output.push(info);
        }
    }
    Ok(output)
}

pub async fn mod_info_parse(source: String, target: Target) -> io::Result<String> {
    let mut content: String;
    let mut strbuf: Vec<u8> = Vec::new();
    let text_file = File::open(source).await;
    let _loading_strbuf = text_file.unwrap().read_to_end(&mut strbuf).await;

    let input: &str = match target {
        Target::Id => "id=",
        Target::Description => "description=",
        Target::Name => "name=",
    };

    content = match str::from_utf8(&strbuf) {
        Ok(content) => content.to_string(),
        Err(_err) => {
            println!("Error reading bytes as utf8");
            "Error".to_string()
        }
    };


    loop {
        if content.contains(&input.to_string()) {
            let offset = content.find('=').unwrap() + 1;
            content.replace_range(..offset, "");
        } else if !content.contains(&input.to_string()) {
            break;
        }
    }

    loop {
        if content.contains("\n") {
            let offset = content.find('\n').unwrap();
            let _ = content.split_off(offset);
        }

        if content.contains("\r") {
            let offset = content.find('\r').unwrap();
            let _ = content.split_off(offset);
        }

        if !content.contains("\n") || !content.contains("\r") {
            break;
        }
    }
    Ok(content)
}

//=== Function for getting Mod Paths ===

pub async fn path_collect(source: &str) -> io::Result<Vec<String>> {
    let mut paths: Vec<String> = Vec::new();

    if let Ok(mut entry) = fs::read_dir(source).await {
        while let Some(dir_entry) = entry.next_entry().await? {
            paths.push(dir_entry.path().to_str().unwrap().to_string());
        }
    }
    Ok(paths)
}

pub async fn path_unwrap<F>(result: F) -> Vec<String> 
where 
    F: Future<Output = std::io::Result<Vec<String>>>,
{
    let vec_result = result.await;
    vec_result.unwrap()
}

//=== Function for getting workshop ids =====

pub async fn work_id_build(source: &str) -> io::Result<Vec<String>> {
    let mut workids: Vec<String> = Vec::new();

    if let Ok(mut entry) = fs::read_dir(source).await {
        while let Some(dir_entry) = entry.next_entry().await? {
            let mut workshop_id = dir_entry
                .path()
                .to_str()
                .unwrap()
                .to_string()
                .replace(source, "");

            if workshop_id.contains("/") || workshop_id.contains("\\") {
                workshop_id = workshop_id.replace("/", "");
                workshop_id = workshop_id.replace("\\", "");
            }
            workids.push(workshop_id);
        }
    }
    println!("work_id_build Sucess!");
    Ok(workids)
}

//=== Functions for recursively locating mod.info directories =====

pub async fn mod_id_path_collecter(source: Vec<String>) -> std::io::Result<Vec<String>> {
    let mut modinfos: Vec<String> = Vec::new();

    for val in source {
        let result = mod_file_finder(val, FileType::ModInfo).await;
        modinfos.push(result);
        
        //let _ = collect_mod_ids(Path::new(&val), &mut modinfos).await;
    }
    println!("mod_id_path_collector Sucess!");
    Ok(modinfos)
}

//=== Functions for recursively locating map names and collecting them =====

pub async fn map_name_collect(source: Vec<String>) -> std::io::Result<Vec<String>> {
    let mut mapnames: Vec<String> = Vec::new();

    for val in source {
        let _ = collect_map_names(Path::new(&val), &mut mapnames).await;
    }

    println!("map_name_collect Sucess!");
    Ok(mapnames)
}

pub async fn collect_map_names(path: &Path, mapnames: &mut Vec<String>) -> std::io::Result<()> {
    if path.is_dir() && let Ok(mut entry) = fs::read_dir(path).await {
            while let Some(dir_entry) = entry.next_entry().await? {
                let next_path = dir_entry.path();

                if next_path.is_dir() && next_path.to_str().unwrap().contains("maps") {
                    if let Ok(mut sub_entry) = fs::read_dir(next_path.clone()).await {
                        while let Some(sub_entry) = sub_entry.next_entry().await? {
                            let mut location = next_path.to_str().unwrap().to_string() + "/";

                            if sub_entry.path().is_dir() {
                                if sub_entry.path().to_str().unwrap().contains("\\") {
                                    location = location.replace("\\", "/"); //<- May be unnecessary
                                }
                                let insertion = sub_entry
                                    .path()
                                    .to_str()
                                    .unwrap()
                                    .to_string()
                                    .replace(&location, "");

                                mapnames.push(insertion);
                            } else {
                                continue;
                            }
                        }
                    }
                } else {
                    let _ = Box::pin(collect_map_names(&next_path, mapnames)).await;
                }
            }
        }
    Ok(())
}

pub async fn names_and_posters(
    initial_path: String,
    workshop_ids: Vec<String>,
) -> Option<HashMap<String, [String; 3]>> {
    let mut output_map: HashMap<String, [String; 3]> = HashMap::new();
    println!("INSIDE OF NAMES AND POSTERS");
    
    for id in workshop_ids {
        let mut values = [id.clone(), String::new(), String::new()];
        let mod_directory = initial_path.clone() + "/" + &id + "/mods/";
        let png_path: String = mod_file_finder(mod_directory.clone(), FileType::Png).await;
        let info_path: String = mod_file_finder(mod_directory.clone(), FileType::ModInfo).await;

        let description: String = match mod_info_parse(info_path.clone(), Target::Description).await {
            Ok(text) => text,
            Err(err) => panic!("{err} Text not located in mod.info"),
        };

        let mod_name: String = match mod_info_parse(info_path, Target::Name).await {
            Ok(text) => text,
            Err(err) => panic!("{err} Text not located in mod.info"),
        };
    
        values[1] = png_path;
        values[2] = description;
    
        output_map.insert(mod_name, values);
    }
    
    println!("{:?}", &output_map);
    
    Some(output_map)
}

pub async fn collect_workshop_ids(workshop_location: String) -> Vec<String> {
    let location = workshop_location;
    let id_vec = work_id_build(&location).await;

    id_vec.unwrap()
}

//TODO: After loading configs and attempting to export, some info values are None
//      Need to backwards investigate why No value is being found
pub async fn collect_selections(
    workshop_location: String,
    filter: HashMap<String, bool>,
    info: HashMap<String, [String; 3]>,
) -> [Vec<String>; 3] {
    let mut workshop_ids: Vec<String> = Vec::new();
    let mut workshop_id_paths: Vec<String> = Vec::new();
    let mut mod_ids: Vec<String> = Vec::new();
    let mut map_ids: Vec<String> = Vec::new();
    let mod_id_locations = match mod_id_path_collecter(workshop_id_paths.clone()).await {
        Ok(output) => output,
        Err(err) => panic!("error getting mod_id file locations {}", err),
    };

    filter.iter().for_each(|(key, value)| {
        if value == &true {
            println!("{:?}", &info.get(&key.clone()));
            workshop_ids.push(info.get(key).unwrap()[0].to_string());
        }
    });

    workshop_ids.iter().for_each(|id| {
        workshop_id_paths.push(format!("{}/{}/", workshop_location, id))
    });

    for mod_info in mod_id_locations.iter() {
        let result = mod_info_parse(mod_info.to_string(), Target::Id).await;
        match result {
            Ok(mod_id) => mod_ids.push(mod_id),
            Err(err) => panic!("issue parsing for mod_id {}", err),
        }
    }

    for mod_directory in workshop_id_paths {
        let result = collect_map_names(std::path::Path::new(&mod_directory), &mut map_ids).await;
        match result {
            Ok(_) => continue,
            Err(err) => panic!("issue parsing for map names {}", err),
        }
    }

    let output = [workshop_ids, mod_ids, map_ids];
    println!("THE OUTPUT {:?}", &output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn does_it_work() {
        let path = "/mnt/d1/SSD1/steamapps/workshop/content/108600/2850935956".to_string();
        let result = mod_file_finder(path, FileType::Png).await;
        println!("\n\n{:?}", result);
    }
}

