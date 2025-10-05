#![windows_subsystem = "windows"]

use iced::alignment::{Horizontal, Vertical};
use iced::event::{self, Status};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Background, Border, Color, Element, Length, Renderer, Task};
use iced_core::{Shadow, Theme, border::Radius};
use std::env::home_dir;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{fs, path};
pub mod explorer;
pub mod localmodinfo;
pub use explorer::*;
pub use localmodinfo::*;

pub static DIRECTORY: Mutex<String> = Mutex::new(String::new());

#[tokio::main]
async fn main() -> iced::Result {
    iced::application("ZSMM", update, view)
        .antialiasing(true)
        .theme(|_s| iced::Theme::KanagawaDragon)
        .run()
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    OpenExplorer,
    ExplorerPathInput(String),
    ExplorerHome,
    ExplorerConfirmPath,
    ExplorerButtonPath(String),
    ExplorerNewPath(PathBuf),
    ExplorerReturn,
    ExplorerExportPath,
    ModInfoCollected(Vec<Vec<String>>),
}

enum State {
    InitialMain,
    LoadedMain,
    FileExplorer,
}

pub struct ZSMM<'a> {
    view: Option<State>,
    file_explorer: Explorer<'a>,
    game_location: Option<String>,
    mod_info: ModInfo,
}

pub struct ModInfo {
    mod_id_vec: Vec<String>,
    workshop_id_vec: Vec<String>,
    map_name_vec: Vec<String>,
}

impl Default for ModInfo {
    fn default() -> Self {
        let state = ModInfo {
            mod_id_vec: Vec::new(),
            workshop_id_vec: Vec::new(),
            map_name_vec: Vec::new(),
        };

        state
    }
}

impl<'a> Default for ZSMM<'a> {
    fn default() -> Self {
        let state = ZSMM {
            view: Some(State::InitialMain),
            file_explorer: Explorer::default(),
            game_location: None,
            mod_info: ModInfo::default(),
        };

        state
    }
}

impl<'a> ZSMM<'a> {
    fn main_view(&self) -> iced::widget::Container<'_, AppMessage> {
        container(row![
            if !self.game_location.is_none() {
                text(format!(
                    "Project Zomboid Located in => {:?}",
                    self.game_location.clone().unwrap()
                ))
            } else {
                text("Please Select Project Zomboid Directory")
            },
            button(text("Select Directory")).on_press(AppMessage::OpenExplorer)
        ])
    }
    fn loaded_view(&self) -> iced::widget::Container<'_, AppMessage> {
        todo!() // Implement a view that displays modinfo
        // Initial View goal -> List on left side with Image and Description on right side
    }
}

fn view<'a>(app: &'a ZSMM) -> Element<'a, AppMessage> {
    match &app.view {
        Some(State::InitialMain) => app.main_view().into(),
        Some(State::LoadedMain) => app.loaded_view().into(),
        Some(State::FileExplorer) => app.file_explorer.explorer_view().into(),
        None => panic!("no view in state!"),
    }
}

fn update<'a>(app: &'a mut ZSMM, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::OpenExplorer => {
            app.view = Some(State::FileExplorer);
            Task::none()
        }
        AppMessage::ExplorerPathInput(string) => {
            app.file_explorer.input_buffer = string;
            println!("Text was input => {}", app.file_explorer.input_buffer);
            Task::none()
        }
        AppMessage::ExplorerButtonPath(string) => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.input_buffer = string;
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
            Task::none()
        }
        AppMessage::ExplorerHome => {
            app.file_explorer.previous_path = PathBuf::new();
            app.file_explorer.current_path = home_dir().unwrap();
            app.file_explorer.input_buffer =
                app.file_explorer.current_path.to_str().unwrap().to_string();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
            Task::none()
        }
        AppMessage::ExplorerNewPath(path_buf) => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.current_path = path_buf;
            app.file_explorer.input_buffer =
                app.file_explorer.current_path.to_str().unwrap().to_string();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
            Task::none()
        }
        AppMessage::ExplorerConfirmPath => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
            Task::none()
        }
        AppMessage::ExplorerReturn => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.return_directory();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
            Task::none()
        }
        AppMessage::ExplorerExportPath => {
            export_directory(
                app.file_explorer
                    .current_path
                    .clone()
                    .to_str()
                    .unwrap()
                    .to_string(),
                &DIRECTORY,
            );
            println!("{:?}", &DIRECTORY.lock().unwrap());
            app.view = Some(State::InitialMain);
            app.game_location = Some(DIRECTORY.lock().unwrap().clone());
            Task::perform(
                collect_ids(app.game_location.clone().unwrap()),
                AppMessage::ModInfoCollected,
            )
        }
        AppMessage::ModInfoCollected(mut vector) => {
            println!("{:?}", &vector);
            app.mod_info.workshop_id_vec = vector.pop().unwrap();
            app.mod_info.map_name_vec = vector.pop().unwrap();
            app.mod_info.mod_id_vec = vector.pop().unwrap();
            app.view = Some(State::LoadedMain);
            Task::none()
        }
    }
}

async fn collect_ids(game_location: String) -> Vec<Vec<String>> {
    let location = game_location;
    let mut output: Vec<Vec<String>> = Vec::new();
    if let Ok(path_vector) = pathcollect(&location).await {
        let id_vec = workidbuild(&location).await;
        let mod_id_path = modidpathcollecter(path_vector.clone()).await;
        let map_name = mapnamecollect(path_vector.clone()).await;
        let mod_ids = id_path_process(mod_id_path.unwrap()).await;

        output.push(mod_ids.unwrap());
        output.push(map_name.unwrap());
        output.push(id_vec.unwrap());
    };

    output
}
