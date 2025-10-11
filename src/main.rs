#![windows_subsystem = "windows"]

#[allow(unused_imports)]
#[allow(unused_import_braces)]
use iced::alignment::{Horizontal, Vertical};
use iced::event::{self, Status};
use iced::widget::{button, checkbox, column, container, image, row, scrollable, text, text_input};
use iced::{Background, Border, Color, Element, Length, Renderer, Task};
use iced_core::Theme;
use std::collections::{HashMap, hash_map::Entry};
use std::env::home_dir;
use std::path::PathBuf;
pub mod explorer;
pub mod localmodinfo;
pub use explorer::*;
pub use localmodinfo::*;

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
    ModInfoCollected(Vec<String>),
    NamesPosters(Option<HashMap<String, [String; 3]>>),
    ModIDChecked(String, bool),
}

enum State {
    InitialMain,
    LoadedMain,
    FileExplorer,
}

pub struct ZSMM<'a> {
    view: Option<State>,
    file_explorer: Explorer<'a>,
    workshop_location: Option<String>,
    mod_info: ModInfo,
    check_state: CheckState,
    selected_mod: SelectedMod,
}

pub struct CheckState {
    num_of_bools: Vec<bool>,
    values: HashMap<String, bool>,
    names_and_details: HashMap<String, [String; 3]>,
    current_bool: String,
}

pub struct ModInfo {
    mod_id_vec: Vec<String>,
    workshop_id_vec: Vec<String>,
    map_name_vec: Vec<String>,
}

pub struct SelectedMod {
    mod_name: String,
    mod_id: String,
    mod_image: String,
    mod_description: String,
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

impl Default for SelectedMod {
    fn default() -> Self {
        let state = SelectedMod {
            mod_name: String::new(),
            mod_id: String::new(),
            mod_image: String::new(),
            mod_description: String::new(),
        };

        state
    }
}

impl Default for CheckState {
    fn default() -> Self {
        CheckState {
            num_of_bools: Vec::new(),
            values: HashMap::new(),
            names_and_details: HashMap::new(),
            current_bool: String::new(),
        }
    }
}

impl<'a> Default for ZSMM<'a> {
    fn default() -> Self {
        let state = ZSMM {
            view: Some(State::InitialMain),
            file_explorer: Explorer::default(),
            workshop_location: None,
            mod_info: ModInfo::default(),
            check_state: CheckState::default(),
            selected_mod: SelectedMod::default(),
        };

        state
    }
}

impl<'a> ZSMM<'a> {
    fn main_view(&self) -> iced::widget::Container<'_, AppMessage> {
        container(row![
            if !self.workshop_location.is_none() {
                text(format!(
                    "Project Zomboid Located in => {:?}",
                    self.workshop_location.clone().unwrap()
                ))
            } else {
                text("Please Select Project Zomboid Directory")
            },
            button(text("Select Directory")).on_press(AppMessage::OpenExplorer)
        ])
    }

    fn checkmark_prep(&mut self) {
        let mut keys: Vec<String> = self
            .check_state
            .names_and_details
            .clone()
            .into_keys()
            .collect();
        let mut current_mod: String = String::new();

        keys.sort();
        self.check_state.num_of_bools = vec![true; keys.len()];

        for (id, truth) in keys.iter().zip(self.check_state.num_of_bools.iter()) {
            if self.check_state.values.is_empty() {
                current_mod = id.clone();
            }
            self.check_state.values.insert(id.to_string(), *truth);
        }

        self.selected_mod = SelectedMod {
            mod_name: current_mod.clone(),
            mod_id: self
                .check_state
                .names_and_details
                .get(&current_mod)
                .unwrap()[0]
                .clone(),
            mod_image: self
                .check_state
                .names_and_details
                .get(&current_mod)
                .unwrap()[1]
                .clone(),
            mod_description: self
                .check_state
                .names_and_details
                .get(&current_mod)
                .unwrap()[2]
                .clone(),
        }
    }

    fn loaded_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let mut mod_col = column![];
        let mut mod_row = row![];

        for (id, truth) in &self.check_state.values {
            mod_row = mod_row.push(
                <iced::widget::Checkbox<'_, AppMessage, Theme, Renderer> as Into<
                    Element<'_, AppMessage, Theme, Renderer>,
                >>::into(
                    checkbox(format!("{}", &id), *truth)
                        .on_toggle(move |truth| AppMessage::ModIDChecked(id.to_string(), truth)),
                ),
            );
            mod_col = mod_col.push(mod_row);
            mod_row = row![];
        }

        container(row![
            column![scrollable(mod_col)],
            column![scrollable(column![
                image(&self.selected_mod.mod_image),
                text(&self.selected_mod.mod_description),
                text(&self.selected_mod.mod_id)
            ])]
        ])
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
            app.workshop_location =
                Some(app.file_explorer.current_path.to_str().unwrap().to_string());
            println!("{:?}", app.workshop_location);
            app.view = Some(State::InitialMain);
            Task::perform(
                collect_workshop_ids(app.workshop_location.clone().unwrap()),
                AppMessage::ModInfoCollected,
            )
        }
        AppMessage::ModInfoCollected(vector) => {
            println!("{:?}", &vector);
            app.mod_info.mod_id_vec = vector;
            Task::perform(
                names_and_posters(
                    app.workshop_location.clone().unwrap(),
                    app.mod_info.mod_id_vec.clone(),
                ),
                AppMessage::NamesPosters,
            )
        }
        AppMessage::NamesPosters(hashmap) => {
            app.check_state.names_and_details = hashmap.unwrap();
            app.checkmark_prep();
            app.view = Some(State::LoadedMain);
            Task::none()
        }
        AppMessage::ModIDChecked(string, _bool) => {
            let current_mod = string.clone();
            app.check_state.current_bool = string.clone();
            app.selected_mod = SelectedMod {
                mod_name: current_mod.clone(),
                mod_id: app.check_state.names_and_details.get(&current_mod).unwrap()[0].clone(),
                mod_image: app.check_state.names_and_details.get(&current_mod).unwrap()[1].clone(),
                mod_description: app.check_state.names_and_details.get(&current_mod).unwrap()[2]
                    .clone(),
            };
            match app.check_state.values.entry(string) {
                Entry::Occupied(mut entry) => match entry.get() {
                    &true => {
                        *entry.get_mut() = false;
                    }
                    &false => {
                        *entry.get_mut() = true;
                    }
                },
                Entry::Vacant(entry) => {
                    entry.insert(true);
                }
            }
            Task::none()
        }
    }
}

async fn collect_workshop_ids(workshop_location: String) -> Vec<String> {
    let location = workshop_location;
    let id_vec = workidbuild(&location).await;

    id_vec.unwrap()
}
