use iced::widget::text;
use std::boxed::Box;
use std::io;
use std::path::Path;
use std::str;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::{fs, test};

//=== Function for getting ModIds =====
pub async fn id_path_process(input_vec: Vec<String>) -> std::io::Result<Vec<String>> {
    let mut output: Vec<String> = Vec::new();
    for info_file in input_vec {
        if let Ok(info) = mod_info_parse(info_file).await {
            output.push(info);
        }
    }
    Ok(output)
}

pub async fn mod_info_parse(source: String) -> io::Result<String> {
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
        if content.contains("id=") {
            let offset = content.find('=').unwrap() + 1;
            content.replace_range(..offset, "");
        } else if !content.contains("id=") {
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
