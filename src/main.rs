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
pub mod corefunctions;
pub mod explorer;
pub use corefunctions::*;
pub use explorer::*;

pub static DIRECTORY: Mutex<String> = Mutex::new(String::new());

#[tokio::main]
async fn main() -> iced::Result {
    application().await
}

async fn application() -> iced::Result {
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
}

enum State {
    Main,
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
            view: Some(State::Main),
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
    async fn collect_ids(&mut self) {
        let location = self.game_location.clone().unwrap();
        if let Ok(path_vector) = pathcollect(&location).await {
            let id_vec = workidbuild(&location).await;
            let mod_id = modidpathcollecter(path_vector.clone()).await;
            let map_name = mapnamecollect(path_vector.clone()).await;

            self.mod_info.mod_id_vec = mod_id.unwrap();
            self.mod_info.map_name_vec = map_name.unwrap();
            self.mod_info.workshop_id_vec = id_vec.unwrap();
        };
    }
}

fn view<'a>(app: &'a ZSMM) -> Element<'a, AppMessage> {
    match &app.view {
        Some(State::Main) => app.main_view().into(),
        Some(State::FileExplorer) => app.file_explorer.explorer_view().into(),
        None => panic!("no view in state!"),
    }
}

async fn update<'a>(app: &'a mut ZSMM<'a>, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::OpenExplorer => {
            app.view = Some(State::FileExplorer);
            Task::none()
        }
        AppMessage::ExplorerPathInput(string) => {
            app.file_explorer.input_buffer = string;
            println!("Text was input => {}", &app.file_explorer.input_buffer);
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
            export_directory(&mut app.file_explorer, &DIRECTORY);
            println!("{:?}", &DIRECTORY.lock().unwrap());
            app.view = Some(State::Main);
            app.game_location = Some(DIRECTORY.lock().unwrap().clone());
            app.collect_ids().await;
            Task::none()
        }
    }
}
