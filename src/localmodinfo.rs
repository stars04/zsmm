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
}

//=== Function for getting ModIds =====
pub async fn id_path_process(input_vec: Vec<String>) -> std::io::Result<Vec<String>> {
    let mut output: Vec<String> = Vec::new();
    for info_file in input_vec {
        if let Ok(info) = mod_info_parse(info_file, Some(Target::Id)).await {
            output.push(info);
        }
    }
    Ok(output)
}

pub async fn mod_info_parse(source: String, target: Option<Target>) -> io::Result<String> {
    let input = match target {
        Some(Target::Id) => "id=",
        Some(Target::Description) => "description=",
        None => panic!("TARGET MISSING"),
    };
    let text_file = File::open(source).await;
    let mut strbuf: Vec<u8> = Vec::new();
    let _loading_strbuf = text_file.unwrap().read_to_end(&mut strbuf).await;
    let mut content = match str::from_utf8(&strbuf) {
        Ok(content) => content.to_string(),
        Err(_err) => {
            println!("Error reading bytes as utf8");
            "Error".to_string()
        }
    };

    loop {
        if content.contains(&format!("{}", input)) {
            let offset = content.find('=').unwrap() + 1;
            content.replace_range(..offset, "");
        } else if !content.contains(&format!("{}", input)) {
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

pub async fn pathcollect(source: &str) -> io::Result<Vec<String>> {
    let mut paths: Vec<String> = Vec::new();

    if let Ok(mut entry) = fs::read_dir(source).await {
        while let Some(dir_entry) = entry.next_entry().await? {
            paths.push(dir_entry.path().to_str().unwrap().to_string());
        }
    }
    Ok(paths)
}

//=== Function for getting workshop ids =====

pub async fn workidbuild(source: &str) -> io::Result<Vec<String>> {
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

pub async fn modidpathcollecter(source: Vec<String>) -> std::io::Result<Vec<String>> {
    let mut modinfos: Vec<String> = Vec::new();

    for val in source {
        let _ = collect_modids(&Path::new(&val), &mut modinfos).await;
    }
    println!("mod_id_path_collector Sucess!");
    Ok(modinfos)
}

pub async fn collect_modids(path: &Path, modinfos: &mut Vec<String>) -> std::io::Result<()> {
    if path.is_dir() {
        if let Ok(mut entry) = fs::read_dir(&path).await {
            while let Some(dir_entry) = entry.next_entry().await? {
                if dir_entry.path().to_str().unwrap().contains("mod.info") {
                    modinfos.push(dir_entry.path().to_str().unwrap().to_string());
                } else if dir_entry.path().is_dir() {
                    let _ = Box::pin(collect_modids(&dir_entry.path(), modinfos)).await;
                }
            }
        }
    }
    Ok(())
}

//=== Functions for recursively locating map names and collecting them =====

pub async fn mapnamecollect(source: Vec<String>) -> std::io::Result<Vec<String>> {
    let mut mapnames: Vec<String> = Vec::new();

    for val in source {
        let _ = collect_mapnames(&Path::new(&val), &mut mapnames).await;
    }

    println!("map_name_collect Sucess!");
    Ok(mapnames)
}

pub async fn collect_mapnames(path: &Path, mapnames: &mut Vec<String>) -> std::io::Result<()> {
    if path.is_dir() {
        if let Ok(mut entry) = fs::read_dir(path).await {
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
                    let _ = Box::pin(collect_mapnames(&next_path, mapnames)).await;
                }
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

    for id in workshop_ids {
        let mut values = [id.clone(), String::new(), String::new()];
        let mod_directory = initial_path.clone() + "/" + &id + "/mods/";

        let mut mod_name_entries = match fs::read_dir(Path::new(&mod_directory)).await {
            Ok(result) => result,
            Err(err) => panic!("{err}"),
        };

        while let Ok(option_submod_name) = mod_name_entries.next_entry().await {
            if let Some(submod_name) = option_submod_name {
                let mut read_next = match fs::read_dir(submod_name.path()).await {
                    Ok(result) => result,
                    Err(err) => panic!("{err}"),
                };

                while let Ok(option_next_file) = read_next.next_entry().await {
                    println!("hey");
                    if let Some(next_file) = option_next_file {
                        let file_path = next_file.path().to_str().unwrap().to_string();
                        let png_path: String;
                        let description: String;

                        if next_file.path().is_file() && file_path.contains(".png") {
                            png_path = next_file.path().to_str().unwrap().into();
                            values[1] = png_path;
                        } else if next_file.path().is_file() && file_path.contains(".info") {
                            description =
                                match mod_info_parse(file_path, Some(Target::Description)).await {
                                    Ok(result) => result,
                                    Err(err) => panic!("{err}"),
                                };
                            values[2] = description;
                        }

                        output_map.insert(
                            submod_name
                                .path()
                                .to_str()
                                .unwrap()
                                .to_string()
                                .replace(&mod_directory, ""),
                            values.clone(),
                        );
                    } else {
                        break;
                    }
                }
            } else {
                break;
            };
        }
    }

    println!("{:?}", &output_map);

    Some(output_map)
}

pub async fn collect_workshop_ids(workshop_location: String) -> Vec<String> {
    let location = workshop_location;
    let id_vec = workidbuild(&location).await;

    id_vec.unwrap()
}

pub async fn collect_selections<'a>(
    workshop_location: String,
    filter: HashMap<String, bool>,
    info: HashMap<String, [String; 3]>,
) -> [Vec<String>; 3] {
    let mut workshop_ids: Vec<String> = Vec::new();
    let mut workshop_id_paths: Vec<String> = Vec::new();
    let mut mod_ids: Vec<String> = Vec::new();
    let mut map_ids: Vec<String> = Vec::new();
    let mod_id_locations: Vec<String>;

    filter.iter().for_each(|(key, value)| {
        if value == &true {
            workshop_ids.push(info.get(key).unwrap()[0].to_string());
        }
    });

    for id in workshop_ids.iter() {
        workshop_id_paths.push(format!("{}/{}/", workshop_location, id))
    }

    mod_id_locations = match modidpathcollecter(workshop_id_paths.clone()).await {
        Ok(output) => output,
        Err(err) => panic!("error getting mod_id file locations {}", err),
    };

    for mod_info in mod_id_locations.iter() {
        println!("{:?}", &mod_info);
        let result = mod_info_parse(mod_info.to_string(), Some(Target::Id)).await;
        match result {
            Ok(mod_id) => mod_ids.push(mod_id),
            Err(err) => panic!("issue parsing for mod_id {}", err),
        }
    }

    for mod_directory in workshop_id_paths {
        let result = collect_mapnames(std::path::Path::new(&mod_directory), &mut map_ids).await;

        match result {
            Ok(_) => continue,
            Err(err) => panic!("issue parsing for map names {}", err),
        }
    }

    [workshop_ids, mod_ids, map_ids]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn does_it_work() {
        let path = String::from("/mnt/d1/SSD1/steamapps/workshop/content/108600/");
        let ids = vec![String::from("2761200458")];
        let result = names_and_posters(path, ids).await;
    }
}
