#[allow(unused_import_braces)]
use crate::AppMessage;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{
    Background, Border, Color, Element, Font, Length, Renderer, Task,
    alignment::{Horizontal, Vertical},
    color,
};
use iced_core::{Shadow, Theme, border::Radius};
use std::env::home_dir;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Explorer<'a> {
    pub os: &'a str,
    pub current_path: PathBuf,
    pub previous_path: PathBuf,
    pub mounted_drives: Vec<String>,
    pub input_buffer: String,
    pub ls_cwd: Vec<String>,
    pub text_options: TextOptions,
}
#[derive(Debug)]
pub struct TextOptions {
    buffer_size: u16,
    ui_size: u16,
}

impl Default for TextOptions {
    fn default() -> Self {
        TextOptions {
            buffer_size: 18,
            ui_size: 18,
        }
    }
}

impl<'a> Default for Explorer<'a> {
    fn default() -> Self {
        let mut state = Explorer {
            os: std::env::consts::OS,
            current_path: home_dir().unwrap(),
            previous_path: PathBuf::new(),
            mounted_drives: Vec::new(),
            input_buffer: String::new(),
            ls_cwd: Vec::new(),
            text_options: TextOptions::default(),
        };

        Explorer::list_directory(
            &mut state,
            Some(home_dir().unwrap().to_str().unwrap().to_string()),
        );
        if state.os != "macos" {
            Explorer::find_mounted_drives(&mut state);
        } else {
            state.mounted_drives.push(String::from("/"));
        }
        state
    }
}

impl<'a> Explorer<'a> {
    pub fn return_directory(&mut self) {
        let mut current_directory = self.current_path.to_str().unwrap().to_string();
        let chop_position = current_directory.rfind("/").unwrap();
        current_directory.replace_range(chop_position.., "");
        if current_directory == "" {
            current_directory = String::from("/")
        }
        self.input_buffer = current_directory;
    }
    pub fn list_directory(&mut self, default: Option<String>) {
        let user_input = match default {
            Some(string) => {
                self.input_buffer = string;
                &self.input_buffer
            }
            None => &self.input_buffer,
        };
        let mut directories: Vec<String> = Vec::new();
        for entry in fs::read_dir(user_input).unwrap() {
            let item = entry.unwrap();
            if item.path().is_dir() {
                directories.push(item.path().to_str().unwrap().to_string())
            } else {
                continue;
            }
        }
        directories.sort();

        self.current_path = self.input_buffer.clone().into();
        self.ls_cwd = directories;
    }

    pub fn directory_explorer(&self) -> iced::widget::Row<'_, AppMessage> {
        let mut directory_viewer = row![];
        let mut directory_column = column![];

        for dirs in self.ls_cwd.clone() {
            directory_column =
                directory_column.push(
                    <iced::widget::Button<'_, AppMessage, Theme, Renderer> as Into<
                        Element<'_, AppMessage, Theme, Renderer>,
                    >>::into(
                        button(text(dirs.replace(
                            &(String::from(self.current_path.to_str().unwrap()) + "/"),
                            "",
                        )))
                        .on_press(AppMessage::ExplorerButtonPath(dirs.to_string())),
                    ),
                );

            let index = self
                .ls_cwd
                .iter()
                .position(|element| *element == dirs)
                .unwrap()
                + 1_usize;
            match self.ls_cwd.len() {
                ..19 => {
                    if index % 9 == 0 {
                        directory_viewer = directory_viewer.push(directory_column);
                        directory_column = column![];
                    } else if dirs == self.ls_cwd[self.ls_cwd.len() - 1] {
                        directory_viewer = directory_viewer.push(directory_column);
                        break; // This else block checks if the element is the last of the vector
                    }
                }
                19.. => {
                    if index % 15 == 0 {
                        directory_viewer = directory_viewer.push(directory_column);
                        directory_column = column![];
                    } else if dirs == self.ls_cwd[self.ls_cwd.len() - 1] {
                        directory_viewer = directory_viewer.push(directory_column);
                        break; // This else block checks if the element is the last of the vector
                    }
                }
            }
        }
        directory_viewer
    }

    fn mounted_view(&self) -> iced::widget::Row<'_, AppMessage> {
        let mut mount_view = row![];
        let mut mount_column = column![text("Mount Points")];
        for mount_point in self.mounted_drives.clone() {
            mount_column = mount_column.push(
                <iced::widget::Button<'_, AppMessage, Theme, Renderer> as Into<
                    Element<'_, AppMessage, Theme, Renderer>,
                >>::into(
                    button(text(mount_point.clone()))
                        .on_press(AppMessage::ExplorerButtonPath(mount_point)),
                ),
            );
        }
        mount_view = mount_view.push(mount_column);
        mount_view
    }
    fn find_mounted_drives(&mut self) {
        let mut output_container: Vec<String> = Vec::new();

        let raw_mounts = match self.os {
            "linux" => Command::new("lsblk").output().expect("Command Success"),
            "windows" => Command::new("wmic logicaldisk get deviceid")
                .output()
                .expect("Success"),
            _ => Command::new("echo error identifying OS")
                .output()
                .expect("Failure"),
        };
        let mut raw_string = String::from_utf8(raw_mounts.stdout).unwrap();
        let target = match self.os {
            "linux" => "/",
            "windows" => ":",
            _ => "Error",
        };
        while raw_string.contains(target) {
            let mut search = raw_string.clone();

            let new_line = search.find("\n").unwrap();

            raw_string.replace_range(..new_line + 1, "");
            search.replace_range(new_line.., "");

            match search.find(target) {
                Some(usize) => {
                    if target == "/" {
                        search.replace_range(..usize, "");
                    } else {
                        search.replace_range(..usize - 1, "");
                    }
                    if !search.contains("/boot") {
                        output_container.push(search.clone());
                    }
                }
                None => {
                    println!("/ not found, continuing search...");
                }
            };
        }
        self.mounted_drives = output_container;
    }
    pub fn explorer_view(&self) -> iced::widget::Container<'_, AppMessage> {
        container(column![
            row![
                column![container(row![
                    button(text("Placeholder").size(self.text_options.ui_size))
                        .on_press(AppMessage::ExplorerNewPath(home_dir().unwrap()))
                ],)],
                column![row![
                    button(text("Home Dir").size(self.text_options.ui_size))
                        .on_press(AppMessage::ExplorerHome),
                    container(
                        text_input(&self.current_path.to_string_lossy(), &self.input_buffer)
                            .width(Length::Fill)
                            .size(self.text_options.buffer_size)
                            .on_input(AppMessage::ExplorerPathInput)
                            .on_submit(AppMessage::ExplorerConfirmPath)
                    )
                    .padding(5),
                    button(text("Back").size(self.text_options.ui_size))
                        .padding(5)
                        .on_press(AppMessage::ExplorerReturn)
                ]]
            ],
            column![row![
                container(self.mounted_view(),)
                    .align_x(Horizontal::Center)
                    .width(Length::Fixed(180.0)),
                scrollable(
                    container(self.directory_explorer())
                        .align_x(Horizontal::Center)
                        .width(Length::Fill)
                )
            ]],
            row![
                container(
                    button(text(format!(
                        "Select Current Directory: {}",
                        &self.current_path.to_string_lossy()
                    )))
                    .on_press(AppMessage::ExplorerExportPath(None))
                )
                .align_x(Horizontal::Right)
                .width(Length::Fill)
            ]
        ])
    }
}
