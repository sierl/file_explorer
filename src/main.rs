mod file_system;
mod look;

use file_system::{get_all_drives, get_files_in_folder, open_file};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{theme, Background, Color, Theme};
use iced::{Alignment, Element, Length, Sandbox, Settings};
// use iced_aw::modal;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::file_system::find_file;

#[derive(Default)]
struct ButtonStyle {
    color: Color,
}

impl button::StyleSheet for ButtonStyle {
    // TODO: what is this `Style` type?
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.color)),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            ..button::Appearance::default()
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    BackPressed,
    ItemPressed { name: String },
    InputChanged(String),
    SearchButtonPressed,
}

struct State {
    path: PathBuf,
    file_names: Vec<String>,
    input_value: String,
    active_file: Option<(usize, String)>, // (index of active file, path to active file)
    last_clicked_time: Instant,
    searching: bool,
}

impl Sandbox for State {
    type Message = Message;

    fn new() -> Self {
        Self {
            path: PathBuf::from("F:\\source\\"),
            file_names: get_files_in_folder(Path::new("F:\\source\\")).unwrap(),
            input_value: String::new(),
            active_file: None,
            last_clicked_time: Instant::now(),
            searching: false,
        }
    }

    fn title(&self) -> String {
        String::from("File Explorer")
    }

    fn view(&self) -> Element<Message> {
        let path_display = text(self.path.display()).width(Length::Fill);

        let search_input = text_input("Search files", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(20);

        let search_button = button("Search")
            .padding(10)
            .on_press(Message::SearchButtonPressed);

        let back_button = button("<-").on_press(Message::BackPressed);

        let files: Column<_> = column(
            self.file_names
                .iter()
                .enumerate()
                .map(|(i, file_name)| {
                    let color = match self.active_file {
                        Some((n, ..)) if n == i => {
                            Color::from_rgba(25.0 / 255.0, 25.0 / 255.0, 25.0 / 255.0, 0.8)
                        }
                        _ => Color::from_rgba(55.0 / 255.0, 55.0 / 255.0, 55.0 / 255.0, 0.8),
                    };

                    // TODO: remove clones
                    button(file_name.as_str())
                        .style(theme::Button::Custom(Box::new(ButtonStyle { color })))
                        .width(Length::Fill)
                        .on_press(Message::ItemPressed {
                            name: file_name.clone(),
                        })
                        .into()
                })
                .collect(),
        )
        .spacing(5)
        .into();

        let content = column![
            path_display,
            row![search_input, search_button]
                .spacing(10)
                .align_items(Alignment::Center),
            back_button,
            files
        ]
        .spacing(25);

        scrollable(
            container(content)
                .width(Length::Fill)
                .padding(40)
                .center_x(),
        )
        .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::BackPressed => {
                println!("back pressed");

                let res = self.path.pop();
                if !res {
                    self.file_names = get_all_drives()
                        .into_iter()
                        .map(|drive| drive.name)
                        .collect();

                    // panic!("path.pop() did nothing");
                } else {
                    // Get files
                    self.file_names = get_files_in_folder(&self.path).unwrap();
                }
            }
            Message::ItemPressed { name } => {
                // double click:
                //       old active item path == new active item path
                //   oor last clicked this item (old active item == new active item)
                // + clicked within an small interval of time (self.last_clicked_interval is small)
                //
                let elapsed_time = self.last_clicked_time.elapsed();

                let new_active_file = self
                    .file_names
                    .iter()
                    .enumerate()
                    .find(|(_, file_name)| **file_name == name);

                // FIXME: do something to avoid this workaround.
                let new_active_file = match new_active_file {
                    Some((index, path)) => Some((index, path.clone())),
                    None => None,
                };

                // FIXME: only path of active file is needed to be compared but we are comparing
                // active index as well as if active file is Some or None.
                if self.active_file == new_active_file && elapsed_time.as_secs_f32() < 0.5 {
                    // double click
                    println!("double clicked on {}", name);

                    // Update path
                    self.path.push(&name);
                    self.active_file = None;

                    if self.path.is_dir() {
                        // Get files
                        self.file_names = get_files_in_folder(&self.path).unwrap();
                    } else {
                        // open file
                        open_file(&self.path).unwrap();
                        if !self.path.pop() {
                            unreachable!();
                        }
                    }
                } else {
                    // single click
                    println!("single click on {}", name);

                    self.active_file = new_active_file;
                    self.last_clicked_time = Instant::now();
                }
            }
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::SearchButtonPressed => {
                println!("Searching for {}", self.input_value);
                self.searching = true;

                let result = find_file(Path::new(&self.input_value), &self.path);

                self.file_names = result
                    .iter()
                    .map(|item| format!("{:?}: {}", item.file_name().unwrap(), item.display()))
                    .collect();
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
        // Theme::custom(theme::Palette {
        //     background: Color::from_rgb(1.0, 0.9, 1.0),
        //     text: Color::BLACK,
        //     primary: Color::from_rgb(0.5, 0.5, 0.0),
        //     success: Color::from_rgb(0.0, 1.0, 0.0),
        //     danger: Color::from_rgb(1.0, 0.0, 0.0),
        // })
    }
}

fn main() -> iced::Result {
    State::run(Settings::default())
}
