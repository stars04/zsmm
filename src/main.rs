#![windows_subsystem = "windows"]

use iced::Length::FillPortion;
#[allow(unused_imports, unused_import_braces)]
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, checkbox, column, container, image, row, scrollable, text, text_input};
#[allow(unused_imports, unused_import_braces)]
use iced::{Background, Border, Color, Element, Length, Renderer, Task};
use iced_core::Theme;
use itertools::izip;
use std::collections::{HashMap, hash_map::Entry};
use std::env::home_dir;
use std::path::PathBuf;
pub mod config;
pub mod explorer;
pub mod localmodinfo;
pub use config::*;
pub use explorer::*;
pub use localmodinfo::*;

#[tokio::main]
async fn main() -> iced::Result {
    iced::application("ZSMM", update, view)
        .antialiasing(true)
        .theme(|_s| iced::Theme::KanagawaDragon)
        .run()
}
//TODO: Eventually optomize the number of message calls for file exploring
//      Implement Messages and state for import/export functionality
#[derive(Debug, Clone)]
pub enum AppMessage {
    Terminal(()),
    GetConfigs,
    LoadOldPath(Option<String>),
    ViewConfigs(Vec<String>),
    LoadConfig(String),
    PreConfigured((Vec<String>,HashMap<String, bool>)),
    Rescan,
    OpenExplorer,
    ExplorerPathInput(String),
    ExplorerHome,
    ExplorerConfirmPath,
    ExplorerButtonPath(String),
    ExplorerNewPath(PathBuf),
    ExplorerReturn,
    ExplorerExportPath(Option<String>),
    ModInfoCollected(Vec<String>),
    NamesPosters(Option<HashMap<String, [String; 3]>>),
    ModIDChecked(String, bool),
    BeginExportSelections,
    FileNameBox(String),
    ExportSelections,
    SelectionsReady([Vec<String>; 3]),
}

enum State {
    InitialMain,
    ConfigLoad,
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
    config_opts: Vec<String>,
    exporting: bool,
    file_name: String,
    output_info: String,
}

#[derive(Default)]
pub struct CheckState {
    num_of_bools: Vec<bool>,
    values: HashMap<String, bool>,
    names_and_details: HashMap<String, [String; 3]>,
    current_bool: String,
}

#[derive(Default)]
pub struct ModInfo {
    mod_id_vec: Vec<String>,
    workshop_id_vec: Vec<String>,
    map_name_vec: Vec<String>,
}

#[derive(Default)]
pub struct SelectedMod {
    mod_name: String,
    mod_id: String,
    mod_image: String,
    mod_description: String,
}


impl<'a> Default for ZSMM<'a> {
    fn default() -> Self {
        ZSMM {
            view: Some(State::InitialMain),
            file_explorer: Explorer::default(),
            workshop_location: None,
            mod_info: ModInfo::default(),
            check_state: CheckState::default(),
            selected_mod: SelectedMod::default(),
            config_opts: Vec::new(),
            exporting: false,
            file_name: String::new(),
            output_info: String::new(),
        }
    }
}

impl<'a> ZSMM<'a> {
    fn intial_view(&self) -> iced::widget::Container<'_, AppMessage> {
        container(row![
            button(text("Load Config")).on_press(AppMessage::GetConfigs),
            button(text("Rescan Mod Folder")).on_press(AppMessage::Rescan),
            button(text("Search for Mods")).on_press(AppMessage::OpenExplorer)
        ])
    }
    fn config_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let mut col = column![];
        let mut row = row![];

        for config in self.config_opts.clone() {
            row = row.push(
                <iced::widget::Button<'_, AppMessage, Theme, Renderer> as Into<
                    Element<'_, AppMessage, Theme, Renderer>,
                >>::into(
                    button(text(config.clone().replace(LIN_CONFIG_LOC, "")))
                    .on_press(AppMessage::LoadConfig(config)))
                );
            
            col = col.push(row);
            row = row![];
        }
        container(
            col
        )
    }
    //TODO: sort ID's
    fn loaded_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let mut mod_col = column![];
        let mut mod_row = row![];

        let mut keys: Vec<String> = self.check_state.values.clone().into_keys().collect();

        keys.sort();
        
        for name in keys {
            let bool = self.check_state.values.get(&name).unwrap();

            mod_row = mod_row.push(
                <iced::widget::Checkbox<'_, AppMessage, Theme, Renderer> as Into<
                    Element<'_, AppMessage, Theme, Renderer>,
                >>::into(
                    checkbox((name).to_string(), *bool)
                        .on_toggle(move |bool| AppMessage::ModIDChecked(name.to_string(), bool)),
                ),
            );
            mod_col = mod_col.push(mod_row);
            mod_row = row![];
        }

        container(column![
            row![
                column![scrollable(mod_col)],
                column![scrollable(column![
                    image(&self.selected_mod.mod_image),
                    text(&self.selected_mod.mod_description),
                    text(&self.selected_mod.mod_id),
                    text(&self.selected_mod.mod_name),
                    button(text("Export Selections")).on_press(AppMessage::ExportSelections)
                ])]
            ]
            .height(FillPortion(15))
            .padding(5),
            row![
                button(text("Export Selections"))
                    .on_press(AppMessage::BeginExportSelections)
                    .padding(2),
                if self.exporting {
                   container(text_input("Enter a File name", &self.file_name)
                       .on_input(AppMessage::FileNameBox)
                       .on_submit(AppMessage::ExportSelections)
                       )
                }else {
                   container(button(text("Import Selections")).padding(2))}
            ]
            .height(FillPortion(1))
            .padding(5),
        ])
    }
    
    fn checkmark_prep(&mut self) {
        let mut current_mod: String = String::new();
        let keys: Vec<String>;

        if self.check_state.values == HashMap::new() { 
            keys = self
                .check_state
                .names_and_details
                .clone()
                .into_keys()
                .collect();
            
            self.check_state.num_of_bools = vec![true; keys.len()];

            for (id, truth) in keys.iter().zip(self.check_state.num_of_bools.iter()) {
                if self.check_state.values.is_empty() {
                    current_mod = id.clone();
                }
                self.check_state.values.insert(id.to_string(), *truth);
            }
        }else {
            keys = self
                .check_state
                .values
                .clone()
                .into_keys()
                .collect();

            current_mod = keys[0].clone();
            
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
}

fn view<'a>(app: &'a ZSMM) -> Element<'a, AppMessage> {
    match &app.view {
        Some(State::InitialMain) => app.intial_view().into(),
        Some(State::ConfigLoad) => app.config_view().into(),
        Some(State::LoadedMain) => app.loaded_view().into(),
        Some(State::FileExplorer) => app.file_explorer.explorer_view().into(),
        None => panic!("no view in state!"),
    }
}

fn update<'a>(app: &'a mut ZSMM, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::Terminal(()) => {
            print!("none");
        },
        AppMessage::GetConfigs => {
            return Task::chain( 
                Task::perform(
                    path_unwrap(path_collect(LIN_CONFIG_LOC)),
                    AppMessage::ViewConfigs),
                Task::perform(
                    load_workshop_location(),
                    AppMessage::LoadOldPath),
                )
        },
        AppMessage::LoadOldPath(string) => {
            app.workshop_location = string;
        }
        AppMessage::ViewConfigs(collection) => {
            app.config_opts = collection;
            app.view = Some(State::ConfigLoad);
        }
        AppMessage::LoadConfig(path) => {
            return Task::perform(
                read_config(path),
                AppMessage::PreConfigured,
            );
        }
        AppMessage::PreConfigured((vector, hashmap)) => {
            app.check_state.values = hashmap;
            return Task::perform(
                pass_to_message(vector),
                AppMessage::ModInfoCollected,    
            );
        }
        AppMessage::Rescan => {
            return Task::perform(
                load_workshop_location(),
                AppMessage::ExplorerExportPath
            )
        },
        AppMessage::OpenExplorer => {
            app.view = Some(State::FileExplorer);
        }
        AppMessage::ExplorerPathInput(string) => {
            app.file_explorer.input_buffer = string;
            println!("Text was input => {}", app.file_explorer.input_buffer);
        }
        AppMessage::ExplorerButtonPath(string) => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.input_buffer = string;
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
        }
        AppMessage::ExplorerHome => {
            app.file_explorer.previous_path = PathBuf::new();
            app.file_explorer.current_path = home_dir().unwrap();
            app.file_explorer.input_buffer =
                app.file_explorer.current_path.to_str().unwrap().to_string();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
        }
        AppMessage::ExplorerNewPath(path_buf) => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.current_path = path_buf;
            app.file_explorer.input_buffer =
                app.file_explorer.current_path.to_str().unwrap().to_string();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
        }
        AppMessage::ExplorerConfirmPath => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
        }
        AppMessage::ExplorerReturn => {
            app.file_explorer.previous_path = app.file_explorer.current_path.clone();
            app.file_explorer.return_directory();
            app.file_explorer.list_directory(None);
            app.file_explorer.directory_explorer();
        }
        AppMessage::ExplorerExportPath(workshop_location) => {
            match workshop_location {
                None => {
                    app.workshop_location =
                    Some(app.file_explorer.current_path.to_str().unwrap().to_string());
                },
                Some(string) => {
                    app.workshop_location = Some(string);
                }
            }
            println!("{:?}", app.workshop_location);
            app.view = Some(State::InitialMain);
            return Task::chain(
                Task::perform(
                    collect_workshop_ids(app.workshop_location.clone().unwrap()),
                    AppMessage::ModInfoCollected,
                ),
                Task::perform(
                    save_workshop_location(app.workshop_location.clone().unwrap()),
                    AppMessage::Terminal,
                )
            )
        }
        AppMessage::ModInfoCollected(vector) => {
            app.mod_info.mod_id_vec = vector;
            return Task::perform(
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
        }
        AppMessage::ModIDChecked(string, _bool) => {
            app.check_state.current_bool = string.clone();
            app.selected_mod = SelectedMod {
                mod_name: string.clone(),
                mod_id: app.check_state.names_and_details.get(&string).unwrap()[0].clone(),
                mod_image: app.check_state.names_and_details.get(&string).unwrap()[1].clone(),
                mod_description: app.check_state.names_and_details.get(&string).unwrap()[2].clone(),
            };
            match app.check_state.values.entry(string) {
                Entry::Occupied(mut entry) => match *entry.get() {
                    true => {
                        *entry.get_mut() = false;
                    }
                    false => {
                        *entry.get_mut() = true;
                    }
                },
                Entry::Vacant(entry) => {
                    entry.insert(true);
                }
            }
        }
        AppMessage::BeginExportSelections => {
            app.exporting = true;
        }
        AppMessage::FileNameBox(file_name) => {
            app.file_name = file_name;
        }
        AppMessage::ExportSelections => {
            app.exporting = false;
            return Task::chain(
                Task::perform(
                    collect_selections(
                        app.workshop_location.clone().unwrap(),
                        app.check_state.values.clone(),
                        app.check_state.names_and_details.clone()
                        ),
                        AppMessage::SelectionsReady
                    ),
                Task::perform(
                    write_selection_config(
                        app.file_name.clone(),
                        app.check_state.values.clone(),
                        app.mod_info.mod_id_vec.clone(),
                    ),
                    AppMessage::Terminal,
                )
            )
        }
        AppMessage::SelectionsReady(output_array) => {
            app.output_info = format_output(output_array);
            println!("{}", app.output_info);
        }
    }
    Task::none()
}

fn format_output(output_array: [Vec<String>; 3]) -> String {
    let mut workshop_ids = String::new();
    let mut mod_ids = String::new();
    let mut map_ids = String::new();

    for (workshop_id, mod_id, map_id) in izip!(&output_array[0], &output_array[1], &output_array[2])
    {
        workshop_ids.push_str(&(workshop_id.to_string() + ","));
        mod_ids.push_str(&(mod_id.to_string() + ","));
        map_ids.push_str(&(map_id.to_string() + ","));
    }

    format!(
        "Workshop Ids\n{}\nMod Ids\n{}\nMap Ids\n{}",
        workshop_ids, mod_ids, map_ids
    )
}

async fn pass_to_message<T>(value: T) -> T {
    value
}
