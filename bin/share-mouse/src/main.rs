use anyhow::Result;
use clap::Parser;
use iced::border::width;
use iced::highlighter;
use std::sync::Arc;
use strum::{Display, EnumString};
use tokio::signal;

use iced::keyboard;
use iced::widget::{
    self, button, center, column, container, horizontal_rule, horizontal_space, pick_list, row, text, text_editor,
    text_input, toggler, tooltip, vertical_rule, vertical_space, Button, Column, Row, Text,
};
use iced::{Center, Element, Fill, Font, Task, Theme};

use std::ffi;
use std::io;
use std::path::{Path, PathBuf};

mod args;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    run().unwrap();

    let args = args::Args::parse();
    let server = Arc::new(server::Server::new(args));
    server.start().await;
    signal::ctrl_c().await?;
    Ok(())
}

pub fn run() -> iced::Result {
    iced::application("🪢 Share My Devices", Application::update, Application::view)
        .theme(Application::theme)
        // .font(include_bytes!("../fonts/icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run_with(Application::new)
}

struct Application {
    theme: Theme,
    page: Page,
    server_listen_port: String,
    client_connect_to: String,
    server_running: bool,
    client_running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
enum Page {
    Hello,
    Client,
    Server,
    MenuOptionRole,
    MenuOptionServer,
    MenuOptionClient,
}

#[derive(Debug, Clone)]
enum Message {
    SaveConfig,
    ConfigSaved(Result<PathBuf, Error>),
    PageChanged(Page),
    ServerPortChanged(String),      // port
    ClientConnectToChanged(String), // ip:port
    StartServer,
    StopServer,
    StartClient,
    StopClient,
    ServerRunning(()),
    ClientRunning(()),
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

impl Application {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::Light,
                page: Page::Hello,
                server_listen_port: "9090".to_owned(),
                client_connect_to: "10.10.10.10:9090".to_owned(),
                server_running: false,
                client_running: false,
            },
            Task::batch([
                // Task::perform(load_file(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR"))), Message::FileOpened),
                widget::focus_next(),
            ]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Message::ActionPerformed(action) => {
            //     self.is_dirty = self.is_dirty || action.is_edit();

            //     self.content.perform(action);

            //     Task::none()
            // }
            // Message::ThemeSelected(theme) => {
            //     self.theme = theme;

            //     Task::none()
            // }
            // Message::WordWrapToggled(word_wrap) => {
            //     self.word_wrap = word_wrap;

            //     Task::none()
            // }
            // Message::NewFile => {
            //     if !self.is_loading {
            //         self.file = None;
            //         self.content = text_editor::Content::new();
            //     }

            //     Task::none()
            // }
            // Message::OpenFile => {
            //     if self.is_loading {
            //         Task::none()
            //     } else {
            //         self.is_loading = true;

            //         // Task::perform(open_file(), Message::FileOpened)
            //     }
            // }
            // Message::FileOpened(result) => {
            //     self.is_loading = false;
            //     self.is_dirty = false;

            //     if let Ok((path, contents)) = result {
            //         self.file = Some(path);
            //         self.content = text_editor::Content::with_text(&contents);
            //     }

            //     Task::none()
            // }
            Message::SaveConfig => {
                Task::none()
                // if self.is_loading {
                //     Task::none()
                // } else {
                //     self.is_loading = true;

                //     Task::perform(save_config(self.file.clone(), self.content.text()), Message::ConfigSaved)
                // }
            }

            Message::ConfigSaved(result) => {
                // self.is_loading = false;

                // if let Ok(path) = result {
                //     self.file = Some(path);
                //     self.is_dirty = false;
                // }

                Task::none()
            }
            Message::StartServer => {
                let server = Arc::new(server::Server::new(args::Args {
                    cmd: args::Commands::Server { server_listen: "0.0.0.0:9090".to_owned() },
                    config_path: "analysis_config.toml".to_owned(),
                }));
                Task::perform(server.start(), Message::ServerRunning)
            }
            Message::StartClient => {
                let client = Arc::new(server::Server::new(args::Args {
                    cmd: args::Commands::Client { connect_to: "ws://192.168.1.25:9090".to_owned() },
                    config_path: "analysis_config.toml".to_owned(),
                }));
                Task::perform(client.start(), Message::ClientRunning)
            }
            Message::StopServer => {
                todo!()
            }
            Message::StopClient => {
                todo!()
            }
            Message::PageChanged(page) => {
                self.page = page;
                Task::none()
            }
            Message::ServerPortChanged(port) => {
                self.server_listen_port = port;
                Task::none()
            }
            Message::ClientConnectToChanged(url) => {
                self.client_connect_to = url;
                Task::none()
            }
            Message::ServerRunning(_) => {
                self.server_running = true;
                Task::none()
            }
            Message::ClientRunning(_) => {
                self.client_running = true;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // 左侧菜单栏
        let menu = Column::new()
            .spacing(7)
            .padding(15)
            .width(250)
            .push(Button::new(Text::new("Role")).on_press(Message::PageChanged(Page::MenuOptionRole)))
            .push(Button::new(Text::new("Server")).on_press(Message::PageChanged(Page::MenuOptionServer)))
            .push(Button::new(Text::new("Client")).on_press(Message::PageChanged(Page::MenuOptionClient)));

        // 右侧内容区域
        let content = match self.page {
            Page::MenuOptionRole => self.view_role_setting(),
            Page::MenuOptionServer => self.view_server_setting(),
            Page::MenuOptionClient => self.view_client_setting(),
            _ => self.view_role_setting(),
        };

        let title = Text::new(self.page.to_string()).height(45);
        let content = column![title, horizontal_rule(10), content];

        row![menu, vertical_rule(10), content].into()
    }

    fn view_role_setting(&self) -> Element<Message> {
        let choose_client_type = column![
            text("Please choose your client type："),
            pick_list([Page::Client, Page::Server], Some(&Page::Server), Message::PageChanged).width(Fill),
        ]
        .spacing(10);

        let content = column![choose_client_type, horizontal_rule(38),].spacing(20).padding(20).max_width(600);

        center(content).into()
    }

    fn view_client_setting(&self) -> Element<Message> {
        let text_input = text_input("Connect to server url... e.g. (127.0.0.1:9090)", &self.server_listen_port)
            .on_input(Message::ClientConnectToChanged)
            .padding(10)
            .size(20);

        let start_client = button(text("Start connect to server...").width(Fill).center())
            .padding(10)
            .on_press(Message::StartClient)
            .style(button::primary);

        let client_config =
            column![text("Server URL"), text_input, start_client].spacing(20).padding(20).max_width(600);

        center(client_config).into()
    }

    fn view_server_setting(&self) -> Element<Message> {
        let text_input = text_input("Serve at port. e.g. (9090)", &self.server_listen_port)
            .on_input(Message::ServerPortChanged)
            .padding(10)
            .size(20);

        let start_server = button(text("Start your server service...").width(Fill).center())
            .padding(10)
            .on_press(Message::StartServer)
            .style(button::primary);

        let server_config =
            column![text("Server Port"), text_input, start_server].spacing(20).padding(20).max_width(600);

        center(server_config).into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).center_x(30));

    if let Some(on_press) = on_press {
        tooltip(action.on_press(on_press), label, tooltip::Position::FollowCursor).style(container::rounded_box).into()
    } else {
        action.style(button::secondary).into()
    }
}

async fn save_config(path: Option<PathBuf>, contents: String) -> Result<PathBuf, Error> {
    todo!()
    // let path = if let Some(path) = path {
    //     path
    // } else {
    //     rfd::AsyncFileDialog::new()
    //         .save_file()
    //         .await
    //         .as_ref()
    //         .map(rfd::FileHandle::path)
    //         .map(Path::to_owned)
    //         .ok_or(Error::DialogClosed)?
    // };

    // tokio::fs::write(&path, contents).await.map_err(|error| Error::IoError(error.kind()))?;

    // Ok(path)
}

// async fn loop_start_server(server: Arc<server::Server>) {
//         server.start().await;
// }

// async fn loop_start_client(client: Arc<server::Server>) {
//     tokio::spawn(async move {
//         client.start().await;
//     });
// }
